use crate::color::Color;
use crate::ray_intersect::Material;

// Valores según la tabla de materiales del proyecto: albedo (peso difuso),
// specular, transparencia y reflectividad.

pub fn water() -> Material {
    Material::new(Color::new(0, 102, 153), 0.6, 0.8, 0.7, 0.9, 1.33)
}

// Opaco y sin brillo: lo que golpea el rayo refractado del agua hacia abajo,
// para que no escape al skybox mirando casi derecho al nadir (ahí la
// proyección equirectangular alía feo, ver commit de la casita/velocidad).
pub fn seafloor() -> Material {
    Material::new(Color::new(15, 55, 65), 0.7, 0.05, 0.0, 0.0, 1.0)
}

pub fn sand() -> Material {
    Material::new(Color::new(204, 178, 128), 0.9, 0.1, 0.0, 0.1, 1.0)
}

pub fn rock() -> Material {
    Material::new(Color::new(128, 128, 128), 0.7, 0.2, 0.0, 0.3, 1.0)
}

pub fn wood() -> Material {
    Material::new(Color::new(153, 102, 51), 0.75, 0.3, 0.0, 0.4, 1.0)
}

pub fn glass() -> Material {
    Material::new(Color::new(179, 204, 230), 0.3, 0.9, 0.95, 0.8, 1.5)
}

// Casita: pared clara y techo a dos aguas en terracota.

pub fn wall() -> Material {
    Material::new(Color::new(235, 230, 215), 0.85, 0.15, 0.0, 0.05, 1.0)
}

pub fn roof() -> Material {
    Material::new(Color::new(178, 60, 40), 0.8, 0.2, 0.0, 0.1, 1.0)
}
