mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod materials;
mod plane;
mod ray_intersect;
mod raytracer;
mod scene;
mod skybox;

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use light::Light;
use nalgebra_glm::Vec3;
use plane::Plane;
use ray_intersect::RayIntersect;
use raylib::consts::KeyboardKey;
use raylib::prelude::RaylibDraw;
use raytracer::{cast_ray, RenderContext};
use scene::Scene;
use skybox::Skybox;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::mpsc;
use std::time::Instant;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const FOV: f32 = PI / 3.0;
const ORBIT_SPEED: f32 = 1.6; // rad/seg
const ZOOM_SPEED: f32 = 6.0; // unidades/seg
const SKYBOX_PATH: &str = "assets/skybox/kiara_1_dawn_2k.hdr";
// 1 rayo cada 4x4 pixeles, sin antialiasing: ~70ms/frame en un i7 con esta
// escena. Es la única calidad que usamos ahora, corriendo sin parar en el
// hilo de fondo (ver comentario en main sobre por qué el render no vive más
// en el loop principal).
// Un ciclo día/noche completo cada ~45 segundos.
const NIGHT_CYCLE_SECONDS: f32 = 45.0;
const LIVE_SAMPLES: u32 = 1;
const LIVE_BLOCK: usize = 4;

/// La parte de la escena que no cambia entre frames: geometría fija, luz y
/// fondo. Se pasan siempre juntos, así que van en un solo parámetro. Son
/// todas referencias (Copy), así que el worker se queda con su propia copia
/// sin tener que clonar la escena. El humo animado y el estado de las
/// ventanas (tecla L) no viven acá porque cambian cada frame — se arman en
/// el loop del worker y se combinan recién al construir el `RenderContext`.
#[derive(Clone, Copy)]
struct SceneView<'a> {
    objects: &'a [Box<dyn RayIntersect>],
    light: &'a Light,
    skybox: &'a Skybox,
}

/// 0.0 = día, 1.0 = noche cerrada, oscilando sin saltos (coseno, arranca en
/// 0.0 = día apenas empieza el programa).
fn night_factor(time: f32) -> f32 {
    0.5 - 0.5 * (time * 2.0 * PI / NIGHT_CYCLE_SECONDS).cos()
}

/// Cubos que cambian de frame a frame: humo + la brasa de la chimenea.
fn dynamic_objects(time: f32, night: f32) -> Vec<Box<dyn RayIntersect>> {
    let mut objects = Scene::smoke_cubes(time);
    objects.push(Scene::chimney_ember(time, night));
    objects
}

fn build_scene() -> Vec<Box<dyn RayIntersect>> {
    let water = materials::water();
    let seafloor = materials::seafloor();

    let mut objects = Scene::create_island_scene().objects;
    objects.push(Box::new(Plane::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        water,
    )));
    // Para que la refracción del agua tenga algo sólido contra qué terminar
    // en vez de escapar al skybox mirando hacia el nadir (ver materials::seafloor).
    objects.push(Box::new(Plane::new(
        Vec3::new(0.0, -2.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        seafloor,
    )));
    objects
}

/// Traza un pixel con supersampling `samples`x`samples` (1 = un solo rayo
/// por el centro del pixel, sin antialiasing).
fn sample_pixel(
    x: usize,
    y: usize,
    samples: u32,
    aspect_ratio: f32,
    tan_fov: f32,
    camera: &Camera,
    ctx: &RenderContext,
) -> u32 {
    let mut r_sum = 0.0;
    let mut g_sum = 0.0;
    let mut b_sum = 0.0;

    for sy in 0..samples {
        for sx in 0..samples {
            let px = x as f32 + (sx as f32 + 0.5) / samples as f32;
            let py = y as f32 + (sy as f32 + 0.5) / samples as f32;

            let screen_x = (2.0 * px / WIDTH as f32 - 1.0) * aspect_ratio * tan_fov;
            let screen_y = (1.0 - 2.0 * py / HEIGHT as f32) * tan_fov;

            let ray_direction = camera.basis_change(&Vec3::new(screen_x, screen_y, -1.0));
            let hex = cast_ray(&camera.eye, &ray_direction, ctx).to_hex();

            r_sum += ((hex >> 16) & 0xFF) as f32;
            g_sum += ((hex >> 8) & 0xFF) as f32;
            b_sum += (hex & 0xFF) as f32;
        }
    }

    let count = (samples * samples) as f32;
    let r = (r_sum / count) as u32;
    let g = (g_sum / count) as u32;
    let b = (b_sum / count) as u32;

    (r << 16) | (g << 8) | b
}

/// Traza la escena y llena el framebuffer, repartiendo filas entre los
/// núcleos disponibles.
///
/// `block_size` > 1 sacrifica resolución por velocidad: calcula un solo rayo
/// cada `block_size`x`block_size` pixeles y repite ese color en el resto del
/// bloque. Con 1 hace el render a resolución completa.
fn render(framebuffer: &mut Framebuffer, camera: &Camera, ctx: &RenderContext, samples: u32, block_size: usize) {
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let tan_fov = (FOV / 2.0).tan();

    let thread_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let rows_per_chunk = HEIGHT.div_ceil(thread_count);

    std::thread::scope(|scope| {
        for (chunk_index, chunk) in framebuffer.buffer.chunks_mut(rows_per_chunk * WIDTH).enumerate() {
            let base_y = chunk_index * rows_per_chunk;

            scope.spawn(move || {
                // Última fila calculada de verdad, reutilizada en las filas
                // intermedias de cada bloque. Se recalcula en la primera fila
                // del chunk (row == 0) para no arrastrar franjas negras entre
                // los rangos de filas de cada hilo.
                let mut key_row = vec![0u32; WIDTH];

                for (row, pixels) in chunk.chunks_mut(WIDTH).enumerate() {
                    let y = base_y + row;

                    if row % block_size == 0 {
                        let mut x = 0;
                        while x < WIDTH {
                            let color = sample_pixel(x, y, samples, aspect_ratio, tan_fov, camera, ctx);
                            let end = (x + block_size).min(WIDTH);
                            key_row[x..end].fill(color);
                            x = end;
                        }
                    }

                    pixels.copy_from_slice(&key_row);
                }
            });
        }
    });
}

/// Aplica input de flechas (orbita), +/- (zoom) y R (reset).
fn handle_camera_input(rl: &raylib::RaylibHandle, camera: &mut Camera, home: &Camera, dt: f32) {
    let orbit_step = ORBIT_SPEED * dt;
    let zoom_step = ZOOM_SPEED * dt;

    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        camera.orbit(orbit_step, 0.0);
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        camera.orbit(-orbit_step, 0.0);
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        camera.orbit(0.0, -orbit_step);
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        camera.orbit(0.0, orbit_step);
    }
    if rl.is_key_down(KeyboardKey::KEY_EQUAL) || rl.is_key_down(KeyboardKey::KEY_KP_ADD) {
        camera.zoom(-zoom_step);
    }
    if rl.is_key_down(KeyboardKey::KEY_MINUS) || rl.is_key_down(KeyboardKey::KEY_KP_SUBTRACT) {
        camera.zoom(zoom_step);
    }
    if rl.is_key_pressed(KeyboardKey::KEY_R) {
        camera.eye = home.eye;
        camera.center = home.center;
        camera.up = home.up;
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Diorama Marino - Raytracing")
        .build();

    rl.set_target_fps(60);

    let home_camera = Camera::new(
        Vec3::new(6.0, 8.7, 16.5),
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let mut camera = Camera::new(home_camera.eye, home_camera.center, home_camera.up);
    // Direccional "desde arriba y desde la derecha": como está lejos
    // respecto al tamaño de la escena (~18 unidades vs. una isla de ~8 de
    // ancho), una luz puntual en esa posición ya aproxima bien una
    // direccional sin agregar un tipo de luz nuevo.
    let light = Light::new(Vec3::new(10.0, 15.0, 6.0), Color::new(255, 255, 255), 1.0);
    let objects = build_scene();
    let skybox = Skybox::load(SKYBOX_PATH);
    let scene = SceneView {
        objects: &objects,
        light: &light,
        skybox: &skybox,
    };
    // Tecla L: prende/apaga W01 y D02 (Material::emissive). Arranca prendido.
    let lights_on = AtomicBool::new(true);
    // Tecla N: salto manual de medio ciclo (día↔noche al toque), en
    // milisegundos para no andar compartiendo un f32 entre hilos. Se suma al
    // tiempo real, así que el ciclo automático sigue corriendo desde ahí.
    let time_offset_ms = AtomicI64::new(0);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let start_time = Instant::now();
    let initial_dynamic = dynamic_objects(0.0, night_factor(0.0));
    render(
        &mut framebuffer,
        &camera,
        &RenderContext {
            objects: scene.objects,
            dynamic_objects: &initial_dynamic,
            light: scene.light,
            skybox: scene.skybox,
            lights_on: true,
            night_factor: night_factor(0.0),
        },
        LIVE_SAMPLES,
        LIVE_BLOCK,
    );

    // El render (incluso en bloques) puede tardar más que un frame de
    // ventana, y con animaciones corriendo hace falta re-trazar todo el
    // tiempo, no solo cuando se mueve la cámara. Si lo hiciéramos en el loop
    // principal, la ventana se congelaría cada vez que tarda. Por eso corre
    // en un hilo aparte que nunca deja de renderizar: el loop principal solo
    // manda la cámara actual y pinta el último frame que haya llegado, así
    // que la ventana siempre responde a los 60 FPS de raylib sin importar
    // cuánto tarde un frame del raytracer.
    let (camera_tx, camera_rx) = mpsc::channel::<Camera>();
    let (frame_tx, frame_rx) = mpsc::channel::<Vec<u32>>();
    // Receiver/Sender no son Sync, así que el closure del worker tiene que
    // ser `move` (se queda dueño de camera_rx/frame_tx). Para que el loop
    // principal pueda seguir usando `lights_on`/`time_offset_ms` después, le
    // pasamos al worker una referencia (sí son Sync, los Atomic* están
    // pensados para compartirse así) en vez de moverlos.
    let lights_on_ref = &lights_on;
    let time_offset_ref = &time_offset_ms;

    std::thread::scope(|thread_scope| {
        thread_scope.spawn(move || {
            let mut worker_framebuffer = Framebuffer::new(WIDTH, HEIGHT);

            while let Ok(mut live_camera) = camera_rx.recv() {
                // Si se acumularon varias cámaras mientras renderizábamos,
                // solo nos importa la más reciente.
                while let Ok(newer) = camera_rx.try_recv() {
                    live_camera = newer;
                }

                let offset = time_offset_ref.load(Ordering::Relaxed) as f32 / 1000.0;
                let time = start_time.elapsed().as_secs_f32() + offset;
                let night = night_factor(time);
                let dynamic = dynamic_objects(time, night);
                let ctx = RenderContext {
                    objects: scene.objects,
                    dynamic_objects: &dynamic,
                    light: scene.light,
                    skybox: scene.skybox,
                    lights_on: lights_on_ref.load(Ordering::Relaxed),
                    night_factor: night,
                };

                render(&mut worker_framebuffer, &live_camera, &ctx, LIVE_SAMPLES, LIVE_BLOCK);

                if frame_tx.send(worker_framebuffer.buffer.clone()).is_err() {
                    break;
                }
            }
        });

        while !rl.window_should_close() {
            let dt = rl.get_frame_time();
            handle_camera_input(&rl, &mut camera, &home_camera, dt);
            if rl.is_key_pressed(KeyboardKey::KEY_L) {
                lights_on.fetch_xor(true, Ordering::Relaxed);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                // Medio ciclo para el lado contrario de donde esté ahora
                // (día->noche o noche->día); el ciclo automático sigue
                // andando desde ese punto.
                let half_cycle_ms = (NIGHT_CYCLE_SECONDS * 500.0) as i64;
                time_offset_ms.fetch_add(half_cycle_ms, Ordering::Relaxed);
            }
            let _ = camera_tx.send(camera);

            if let Ok(mut latest) = frame_rx.try_recv() {
                while let Ok(newer) = frame_rx.try_recv() {
                    latest = newer;
                }
                framebuffer.buffer = latest;
            }

            let mut d = rl.begin_drawing(&thread);
            d.clear_background(raylib::color::Color::BLACK);

            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let hex = framebuffer.buffer[y * WIDTH + x];
                    let r = ((hex >> 16) & 0xFF) as u8;
                    let g = ((hex >> 8) & 0xFF) as u8;
                    let b = (hex & 0xFF) as u8;
                    d.draw_pixel(x as i32, y as i32, raylib::color::Color::new(r, g, b, 255));
                }
            }
        }

        // Cierra el canal para que el worker salga de su loop y el scope
        // pueda esperar a que termine antes de volver de main().
        drop(camera_tx);
    });
}
