use crate::color::Color;
use crate::light::Light;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::skybox::Skybox;
use nalgebra_glm::{dot, Vec3};

// Más alto que un valor "realista": con una sola luz y sin rebotes, las
// caras sin luz directa quedaban casi negras y se veía muy duro.
const AMBIENT_FACTOR: f32 = 0.35;
const SHININESS: f32 = 32.0;
const MAX_DEPTH: u32 = 4;
const BIAS: f32 = 1e-3;
const EMISSIVE_INTENSITY: f32 = 1.5;
const TILE_SIZE: f32 = 0.3;
const TILE_COLOR_A: Color = Color::from_hex(0xB33A28);
const TILE_COLOR_B: Color = Color::from_hex(0x9A2F20);

// Tamaño angular (radianes) de cada celda de la grilla de estrellas: a este
// FOV (60°) cada pixel mide ~0.0008 rad, así que 0.004 da estrellas de
// apenas unos pixeles en vez de los bloques gigantes de la primera prueba.
const STAR_CELL: f32 = 0.004;
const STAR_DENSITY: f32 = 0.9965; // umbral del hash: más alto = menos estrellas
const STAR_COLOR: Color = Color::new(255, 255, 240);
// Las estrellas de verdad no tienen por qué verse más tenues, pero su
// reflejo en el agua se veía como un cuadradito sólido y duro; bajar la
// opacidad global las hace leer más translúcidas ahí sin tocar el reflejo
// en sí (el agua ya mezcla por su cuenta).
const STAR_OPACITY: f32 = 0.55;
const MOON_RADIUS: f32 = 0.0015; // 1 - cos(radio angular) ≈ 3°, no 12° como antes
const MOON_COLOR: Color = Color::new(230, 230, 215);

/// Todo lo que necesita un rayo aparte de su origen/dirección. `objects` es
/// la escena estática; `dynamic_objects` son los cubos que cambian cada
/// frame (el humo animado) — van separados porque se reconstruyen en cada
/// render en vez de vivir en la escena fija.
pub struct RenderContext<'a> {
    pub objects: &'a [Box<dyn RayIntersect>],
    pub dynamic_objects: &'a [Box<dyn RayIntersect>],
    pub light: &'a Light,
    pub skybox: &'a Skybox,
    pub lights_on: bool,
    /// 0.0 = pleno día (el skybox tal cual, Kiara Dawn), 1.0 = noche cerrada.
    /// Oscurece el cielo/ambient y atenúa el sol; nunca llega a negro total
    /// para no perder lectura de la escena.
    pub night_factor: f32,
}

fn closest_intersect(ray_origin: &Vec3, ray_direction: &Vec3, ctx: &RenderContext) -> Option<Intersect> {
    ctx.objects
        .iter()
        .chain(ctx.dynamic_objects.iter())
        .filter_map(|object| object.ray_intersect(ray_origin, ray_direction))
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
}

/// Patrón de tejas alternadas: ver INSTRUCCIONES_ISLA.md sección 5.
fn tile_color(point: &Vec3) -> Color {
    let cell = (point.x / TILE_SIZE).floor() + (point.y / TILE_SIZE).floor() + (point.z / TILE_SIZE).floor();
    if (cell as i64).rem_euclid(2) == 0 {
        TILE_COLOR_A
    } else {
        TILE_COLOR_B
    }
}

/// Sample del skybox atenuado por el ciclo día/noche.
fn sky(ctx: &RenderContext, direction: &Vec3) -> Color {
    ctx.skybox.sample(direction) * (1.0 - ctx.night_factor * 0.85)
}

/// Hash determinístico de dos enteros a [0, 1). No necesita ser criptográfico,
/// solo parecer aleatorio y no repetirse en una grilla chica.
fn hash2(x: i32, y: i32) -> f32 {
    let mut h = (x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263)) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}

/// Estrellas procedurales (no vienen en el panorama HDR) y una luna: una
/// grilla en coordenadas esféricas con un hash por celda decide si esa
/// celda tiene estrella, y un disco fijo hace de luna. Solo se usa para el
/// cielo "de fondo" (rayos que escapan sin golpear nada) — si se usara
/// también para el ambient de las superficies, los puntitos de estrella se
/// colarían como ruido en la luz difusa.
fn sky_background(ctx: &RenderContext, direction: &Vec3) -> Color {
    let base = sky(ctx, direction);

    // Antes del atardecer ni se notarían; después de medianoche se vuelven
    // a apagar hacia el amanecer.
    let visibility = ((ctx.night_factor - 0.5) * 2.0).clamp(0.0, 1.0);
    if visibility <= 0.0 {
        return base;
    }

    // Alineada con hacia dónde mira la cámara inicial para que la luna caiga
    // dentro de cuadro de entrada.
    let moon_dir = Vec3::new(-0.3, 0.3, -0.9).normalize();
    if dot(direction, &moon_dir) > 1.0 - MOON_RADIUS {
        return base + MOON_COLOR * visibility;
    }

    let azimuth = direction.z.atan2(direction.x);
    let elevation = direction.y.asin();
    let cell_x = (azimuth / STAR_CELL).floor() as i32;
    let cell_y = (elevation / STAR_CELL).floor() as i32;
    let twinkle = hash2(cell_x, cell_y);
    if twinkle > STAR_DENSITY {
        let brightness = (twinkle - STAR_DENSITY) / (1.0 - STAR_DENSITY);
        return base + STAR_COLOR * (brightness * visibility * STAR_OPACITY);
    }

    base
}

fn in_shadow(point: &Vec3, light_dir: &Vec3, light_distance: f32, ctx: &RenderContext) -> bool {
    let shadow_origin = point + light_dir * BIAS;
    closest_intersect(&shadow_origin, light_dir, ctx).is_some_and(|hit| hit.distance < light_distance)
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * dot(incident, normal) * normal
}

/// Snell's law. `normal` debe apuntar del lado de donde viene el rayo.
/// En incidencia rasante o índices degenerados puede haber reflexión total
/// interna: en ese caso devuelve la reflexión en vez de un rayo inválido.
fn refract(incident: &Vec3, normal: &Vec3, ior: f32) -> Vec3 {
    let mut cos_i = dot(incident, normal).clamp(-1.0, 1.0);
    let (n, eta) = if cos_i < 0.0 {
        (*normal, 1.0 / ior) // entra al material
    } else {
        cos_i = -cos_i;
        (-normal, ior) // sale del material
    };

    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        reflect(incident, normal)
    } else {
        (eta * incident + (eta * -cos_i - k.sqrt()) * n).normalize()
    }
}

fn cast_ray_recursive(ray_origin: &Vec3, ray_direction: &Vec3, ctx: &RenderContext, depth: u32) -> Color {
    if depth >= MAX_DEPTH {
        return sky_background(ctx, ray_direction);
    }

    let Some(intersect) = closest_intersect(ray_origin, ray_direction, ctx) else {
        return sky_background(ctx, ray_direction);
    };

    let material = intersect.material;

    // Ventanas encendidas: color plano sin depender de luz ni sombras (ver
    // Material::emissive). Apagadas, caen directo al shading normal de abajo
    // con su mismo color y las propiedades físicas de Vidrio.
    if let Some(glow) = material.emissive {
        if ctx.lights_on {
            return glow * EMISSIVE_INTENSITY;
        }
    }

    let base_color = if material.tiled { tile_color(&intersect.point) } else { material.diffuse };

    let light_vector = ctx.light.position - intersect.point;
    let light_distance = light_vector.magnitude();
    let light_dir = light_vector / light_distance;
    let view_dir = -ray_direction.normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let diffuse_intensity = dot(&intersect.normal, &light_dir).max(0.0);
    let specular_intensity = dot(&view_dir, &reflect_dir).max(0.0).powf(SHININESS);

    // Si la cara ya mira para otro lado la luz no aporta nada: no hace falta
    // ni tirar el rayo de sombra.
    let lit = diffuse_intensity > 0.0 && !in_shadow(&intersect.point, &light_dir, light_distance, ctx);
    let shadow_factor = if lit { 1.0 } else { 0.0 };

    // Sesgado hacia arriba en vez de la normal exacta: si no, caras que
    // miran casi al horizonte samplean la silueta oscura de las montañas
    // del panorama y quedan negras sin importar AMBIENT_FACTOR.
    let ambient_dir = (intersect.normal + Vec3::new(0.0, 1.5, 0.0)).normalize();
    let ambient = base_color * sky(ctx, &ambient_dir) * AMBIENT_FACTOR;

    // El sol también se apaga de noche, no solo el cielo — si no, la escena
    // se oscurece pero sigue con sombras duras de mediodía.
    let sun_factor = ctx.light.intensity * shadow_factor * (1.0 - ctx.night_factor * 0.8);
    let diffuse = base_color * ctx.light.color * (material.albedo * diffuse_intensity * sun_factor);
    let specular = ctx.light.color * (material.specular * specular_intensity * sun_factor);

    let local_weight = (1.0 - material.reflectivity - material.transparency).max(0.0);
    let mut color = (ambient + diffuse + specular) * local_weight;

    if material.reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &intersect.normal);
        let reflect_origin = intersect.point + reflect_direction * BIAS;
        let reflect_color = cast_ray_recursive(&reflect_origin, &reflect_direction, ctx, depth + 1);
        color = color + reflect_color * material.reflectivity;
    }

    if material.transparency > 0.0 {
        let refract_direction = refract(ray_direction, &intersect.normal, material.ior);
        let refract_origin = intersect.point + refract_direction * BIAS;
        let refract_color = cast_ray_recursive(&refract_origin, &refract_direction, ctx, depth + 1);
        color = color + refract_color * material.transparency;
    }

    color
}

pub fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, ctx: &RenderContext) -> Color {
    cast_ray_recursive(ray_origin, ray_direction, ctx, 0)
}
