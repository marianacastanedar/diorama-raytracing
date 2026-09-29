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
use raytracer::cast_ray;
use scene::Scene;
use skybox::Skybox;
use std::f32::consts::PI;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const FOV: f32 = PI / 3.0;
const ORBIT_SPEED: f32 = 1.6; // rad/seg
const ZOOM_SPEED: f32 = 6.0; // unidades/seg
const SKYBOX_PATH: &str = "assets/skybox/kiara_1_dawn_2k.hdr";
const PREVIEW_SAMPLES: u32 = 1; // mientras se mueve la cámara: 1 rayo/pixel
const FINAL_SAMPLES: u32 = 2; // en reposo: supersampling 2x2 para antialiasing

fn build_scene() -> Vec<Box<dyn RayIntersect>> {
    let water = materials::water();

    let mut objects = Scene::create_island_scene().objects;
    objects.push(Box::new(Plane::new(
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        water,
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
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    skybox: &Skybox,
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
            let hex = cast_ray(&camera.eye, &ray_direction, objects, light, skybox).to_hex();

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
/// núcleos disponibles: en single-thread cada frame tarda ~200ms, suficiente
/// para que mover la cámara se sienta trabado.
fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    skybox: &Skybox,
    samples: u32,
) {
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
                for (row, pixels) in chunk.chunks_mut(WIDTH).enumerate() {
                    let y = base_y + row;

                    for (x, pixel) in pixels.iter_mut().enumerate() {
                        *pixel = sample_pixel(
                            x, y, samples, aspect_ratio, tan_fov, camera, objects, light, skybox,
                        );
                    }
                }
            });
        }
    });
}

/// Aplica input de flechas (orbita), +/- (zoom) y R (reset). Devuelve true si
/// la cámara cambió, para no re-trazar la escena cuando el usuario no toca nada.
fn handle_camera_input(rl: &raylib::RaylibHandle, camera: &mut Camera, home: &Camera, dt: f32) -> bool {
    let mut changed = false;
    let orbit_step = ORBIT_SPEED * dt;
    let zoom_step = ZOOM_SPEED * dt;

    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        camera.orbit(orbit_step, 0.0);
        changed = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        camera.orbit(-orbit_step, 0.0);
        changed = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        camera.orbit(0.0, -orbit_step);
        changed = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        camera.orbit(0.0, orbit_step);
        changed = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_EQUAL) || rl.is_key_down(KeyboardKey::KEY_KP_ADD) {
        camera.zoom(-zoom_step);
        changed = true;
    }
    if rl.is_key_down(KeyboardKey::KEY_MINUS) || rl.is_key_down(KeyboardKey::KEY_KP_SUBTRACT) {
        camera.zoom(zoom_step);
        changed = true;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_R) {
        camera.eye = home.eye;
        camera.center = home.center;
        camera.up = home.up;
        changed = true;
    }

    changed
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Diorama Marino - Raytracing")
        .build();

    rl.set_target_fps(60);

    let home_camera = Camera::new(
        Vec3::new(7.0, 6.0, 9.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let mut camera = Camera::new(home_camera.eye, home_camera.center, home_camera.up);
    let light = Light::new(Vec3::new(4.0, 8.0, 6.0), Color::new(255, 255, 255), 1.0);
    let objects = build_scene();
    let skybox = Skybox::load(SKYBOX_PATH);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    render(&mut framebuffer, &camera, &objects, &light, &skybox, FINAL_SAMPLES);
    let mut was_moving = false;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let moving = handle_camera_input(&rl, &mut camera, &home_camera, dt);

        if moving {
            // Preview rápido mientras se mueve la cámara.
            render(&mut framebuffer, &camera, &objects, &light, &skybox, PREVIEW_SAMPLES);
        } else if was_moving {
            // Se acaba de soltar: una pasada final con antialiasing.
            render(&mut framebuffer, &camera, &objects, &light, &skybox, FINAL_SAMPLES);
        }
        was_moving = moving;

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
}
