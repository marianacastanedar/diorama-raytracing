use crate::color::Color;
use crate::ray_intersect::Material;

// Valores según la tabla de materiales del proyecto: albedo (peso difuso),
// specular, transparencia y reflectividad.

pub fn water() -> Material {
    Material::new(Color::new(0, 102, 153), 0.6, 0.8, 0.7, 0.9)
}

pub fn sand() -> Material {
    Material::new(Color::new(204, 178, 128), 0.9, 0.1, 0.0, 0.1)
}

pub fn rock() -> Material {
    Material::new(Color::new(128, 128, 128), 0.7, 0.2, 0.0, 0.3)
}

pub fn wood() -> Material {
    Material::new(Color::new(153, 102, 51), 0.75, 0.3, 0.0, 0.4)
}

pub fn glass() -> Material {
    Material::new(Color::new(179, 204, 230), 0.3, 0.9, 0.95, 0.8)
}
