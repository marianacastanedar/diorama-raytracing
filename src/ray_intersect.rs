use crate::color::Color;
use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub albedo: f32,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: f32,
}

impl Material {
    pub fn new(
        diffuse: Color,
        albedo: f32,
        specular: f32,
        transparency: f32,
        reflectivity: f32,
        ior: f32,
    ) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
            transparency,
            reflectivity,
            ior,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub material: Material,
}

// Sync: el render reparte los objetos de la escena entre varios hilos.
pub trait RayIntersect: Sync {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}
