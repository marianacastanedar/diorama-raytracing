use crate::color::Color;
use crate::light::Light;
use crate::ray_intersect::{Intersect, RayIntersect};
use nalgebra_glm::{dot, Vec3};

const SKY_COLOR: Color = Color::new(30, 30, 45);
const AMBIENT_FACTOR: f32 = 0.1;
const SHININESS: f32 = 32.0;

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

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
) -> Color {
    let Some(intersect) = closest_intersect(ray_origin, ray_direction, objects) else {
        return SKY_COLOR;
    };

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = -ray_direction.normalize();
    let reflect_dir = 2.0 * dot(&intersect.normal, &light_dir) * intersect.normal - light_dir;

    let diffuse_intensity = dot(&intersect.normal, &light_dir).max(0.0);
    let specular_intensity = dot(&view_dir, &reflect_dir).max(0.0).powf(SHININESS);

    let ambient = intersect.material.diffuse * AMBIENT_FACTOR;
    let diffuse = intersect.material.diffuse
        * light.color
        * (intersect.material.albedo * diffuse_intensity * light.intensity);
    let specular = light.color * (intersect.material.specular * specular_intensity * light.intensity);

    ambient + diffuse + specular
}
