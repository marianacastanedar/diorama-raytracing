use crate::color::Color;
use crate::ray_intersect::Material;

// Valores según la tabla de materiales del proyecto: albedo (peso difuso),
// specular, transparencia y reflectividad. El color de cada cubo lo decide
// quien arma la escena (ver scene.rs), así que estas funciones lo reciben
// como parámetro en vez de traerlo fijo.

// Menos reflectividad y más transparencia que un espejo puro: se alcanza a
// ver el fondo marino a través del agua.
pub fn water() -> Material {
    Material::new(Color::new(25, 130, 155), 0.5, 0.8, 0.8, 0.5, 1.33)
}

// Opaco y sin brillo: lo que golpea el rayo refractado del agua hacia abajo,
// para que no escape al skybox mirando casi derecho al nadir (ahí la
// proyección equirectangular alía feo).
pub fn seafloor() -> Material {
    Material::new(Color::new(20, 70, 85), 0.7, 0.05, 0.0, 0.0, 1.0)
}

pub fn sand(color: Color) -> Material {
    Material::new(color, 0.9, 0.1, 0.0, 0.1, 1.0)
}

pub fn rock(color: Color) -> Material {
    Material::new(color, 0.7, 0.2, 0.0, 0.25, 1.0)
}

pub fn wood(color: Color) -> Material {
    Material::new(color, 0.75, 0.3, 0.0, 0.3, 1.0)
}

pub fn glass(color: Color) -> Material {
    Material::new(color, 0.3, 0.9, 0.95, 0.8, 1.5)
}

/// "Teja": Arena con el patrón procedural alternado calculado desde el punto
/// de impacto (ver `raytracer::tile_color`) en vez de un color fijo.
pub fn tile(color: Color) -> Material {
    sand(color).tiled()
}
