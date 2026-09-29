use crate::color::Color;
use crate::light::Light;
use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::{dot, Vec3};

const SKY_COLOR: Color = Color::new(30, 30, 45);
const AMBIENT_FACTOR: f32 = 0.1;
const SHININESS: f32 = 32.0;
const MAX_DEPTH: u32 = 4;
const BIAS: f32 = 1e-3;

fn closest_intersect(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
) -> Option<Intersect> {
    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            if closest.is_none_or(|c| intersect.distance < c.distance) {
                closest = Some(intersect);
            }
        }
    }

    closest
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

fn cast_ray_recursive(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    depth: u32,
) -> Color {
    if depth >= MAX_DEPTH {
        return SKY_COLOR;
    }

    let Some(intersect) = closest_intersect(ray_origin, ray_direction, objects) else {
        return SKY_COLOR;
    };

    let material = intersect.material;

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = -ray_direction.normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let diffuse_intensity = dot(&intersect.normal, &light_dir).max(0.0);
    let specular_intensity = dot(&view_dir, &reflect_dir).max(0.0).powf(SHININESS);

    let ambient = material.diffuse * AMBIENT_FACTOR;
    let diffuse =
        material.diffuse * light.color * (material.albedo * diffuse_intensity * light.intensity);
    let specular = light.color * (material.specular * specular_intensity * light.intensity);

    let local_weight = (1.0 - material.reflectivity - material.transparency).max(0.0);
    let mut color = (ambient + diffuse + specular) * local_weight;

    if material.reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &intersect.normal);
        let reflect_origin = intersect.point + reflect_direction * BIAS;
        let reflect_color =
            cast_ray_recursive(&reflect_origin, &reflect_direction, objects, light, depth + 1);
        color = color + reflect_color * material.reflectivity;
    }

    if material.transparency > 0.0 {
        let refract_direction = refract(ray_direction, &intersect.normal, material.ior);
        let refract_origin = intersect.point + refract_direction * BIAS;
        let refract_color =
            cast_ray_recursive(&refract_origin, &refract_direction, objects, light, depth + 1);
        color = color + refract_color * material.transparency;
    }

    color
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
) -> Color {
    cast_ray_recursive(ray_origin, ray_direction, objects, light, 0)
}
