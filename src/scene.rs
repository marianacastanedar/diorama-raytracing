use crate::color::Color;
use crate::cube::Cube;
use crate::materials;
use crate::ray_intersect::{Material, RayIntersect};
use nalgebra_glm::Vec3;
use std::f32::consts::PI;

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { objects: Vec::new() }
    }

    /// Isla flotante sobre el agua: montículo rocoso con puntas hacia abajo,
    /// casa de dos secciones con techo de tejas y balcón, árbol y arbustos.
    /// Coordenadas, tamaños y colores tal como vienen de INSTRUCCIONES_ISLA.md
    /// (sección 4) — no son valores "a ojo", así que se transcriben
    /// literalmente en vez de generarlos con loops. El humo (sección 4.7) es
    /// aparte, en `smoke_cubes`, porque se anima con el tiempo.
    pub fn create_island_scene() -> Self {
        let mut scene = Scene::new();

        // 4.1 Isla (11 cubos). La superficie del pasto queda en y=4.0; la
        // punta más baja (I10) llega a y=0.4, dejando un hueco sobre el agua
        // (que está en y=0) donde se ve el reflejo.
        scene.push_box((0.0, 3.9, 0.0), (8.0, 0.2, 6.0), materials::sand(Color::from_hex(0x6E9B3F)));
        scene.push_box((0.0, 3.35, 0.0), (7.6, 0.9, 5.6), materials::rock(Color::from_hex(0xC27A3E)));
        scene.push_box((0.2, 2.5, 0.0), (6.0, 0.8, 4.4), materials::rock(Color::from_hex(0xB86F36)));
        scene.push_box((0.4, 1.75, 0.0), (4.2, 0.7, 3.2), materials::rock(Color::from_hex(0xAD6430)));
        scene.push_box((0.6, 1.15, 0.0), (2.4, 0.5, 2.0), materials::rock(Color::from_hex(0xA25B2C)));
        scene.push_box((-1.8, 1.7, 0.6), (1.2, 0.8, 1.0), materials::rock(Color::from_hex(0xB86F36)));
        scene.push_box((-1.9, 1.1, 0.6), (0.7, 0.6, 0.6), materials::rock(Color::from_hex(0xA25B2C)));
        scene.push_box((2.6, 1.9, -0.8), (1.0, 0.8, 1.0), materials::rock(Color::from_hex(0xB86F36)));
        scene.push_box((2.7, 1.3, -0.8), (0.6, 0.5, 0.6), materials::rock(Color::from_hex(0xA25B2C)));
        scene.push_box((0.6, 0.65, 0.0), (1.0, 0.5, 0.9), materials::rock(Color::from_hex(0x8F4F26)));
        scene.push_box((-0.8, 1.2, -1.2), (0.8, 0.7, 0.8), materials::rock(Color::from_hex(0xA25B2C)));

        // 4.2 Casa (10 cubos): torre + anexo, techos a dos aguas de tejas
        // (material Arena con el patrón de `materials::tile`), chimenea y
        // toldo de la puerta.
        let tile = Color::from_hex(0xB33A28);
        scene.push_box((0.8, 6.0, -0.5), (2.0, 4.0, 2.0), materials::rock(Color::from_hex(0xA8A296)));
        scene.push_box((-1.05, 5.0, 0.5), (1.7, 2.0, 2.2), materials::sand(Color::from_hex(0xE2A878)));
        scene.push_box((0.8, 8.15, -0.5), (2.4, 0.3, 2.4), materials::tile(tile));
        scene.push_box((0.8, 8.45, -0.5), (1.8, 0.3, 2.4), materials::tile(tile));
        scene.push_box((0.8, 8.75, -0.5), (1.2, 0.3, 2.4), materials::tile(tile));
        scene.push_box((0.8, 9.05, -0.5), (0.6, 0.3, 2.4), materials::tile(tile));
        scene.push_box((-1.05, 6.15, 0.5), (2.0, 0.3, 2.6), materials::tile(tile));
        scene.push_box((-0.75, 6.45, 0.5), (1.4, 0.3, 2.6), materials::tile(tile));
        scene.push_box((1.4, 9.0, -1.0), (0.4, 1.2, 0.4), materials::rock(Color::from_hex(0xD98A55)));
        scene.push_box((-1.05, 5.45, 1.75), (1.0, 0.1, 0.35), materials::tile(tile));

        // 4.3 Puertas y ventanas (9 cubos). W01 y D02 son las ventanas que
        // la tecla L prende/apaga (ver `Material::emissive`); apagadas se
        // comportan como vidrio normal con su mismo color.
        let emissive_glow = Color::from_hex(0xFFD97A);
        scene.push_box(
            (0.8, 4.9, 0.52),
            (0.7, 0.6, 0.06),
            materials::glass(emissive_glow).emissive(emissive_glow),
        );
        scene.push_box((0.8, 6.6, 0.52), (0.5, 1.0, 0.06), materials::wood(Color::from_hex(0x7A4A2A)));
        scene.push_box((0.8, 7.15, 0.52), (0.3, 0.1, 0.06), materials::wood(Color::from_hex(0x7A4A2A)));
        scene.push_box((-0.22, 6.5, -0.1), (0.06, 1.0, 0.3), materials::glass(Color::from_hex(0x2F4A3A)));
        scene.push_box((-0.22, 6.5, -0.9), (0.06, 1.0, 0.3), materials::glass(Color::from_hex(0x2F4A3A)));
        scene.push_box((-1.92, 4.9, 0.5), (0.06, 0.7, 0.3), materials::glass(Color::from_hex(0x2F4A3A)));
        scene.push_box((-1.05, 4.6, 1.62), (0.6, 1.2, 0.06), materials::wood(Color::from_hex(0xA66A3A)));
        scene.push_box(
            (-1.05, 4.95, 1.66),
            (0.35, 0.25, 0.04),
            materials::glass(emissive_glow).emissive(emissive_glow),
        );
        scene.push_box((-1.05, 4.06, 1.95), (1.2, 0.12, 0.6), materials::rock(Color::from_hex(0x3A3A3A)));

        // 4.4 Balcón (8 cubos, todos Madera #4A3426).
        let balcony_wood = Color::from_hex(0x4A3426);
        scene.push_box((0.8, 6.0, 0.8), (1.4, 0.1, 0.6), materials::wood(balcony_wood));
        scene.push_box((0.8, 6.45, 1.07), (1.4, 0.06, 0.06), materials::wood(balcony_wood));
        scene.push_box((0.12, 6.45, 0.8), (0.06, 0.06, 0.6), materials::wood(balcony_wood));
        scene.push_box((1.48, 6.45, 0.8), (0.06, 0.06, 0.6), materials::wood(balcony_wood));
        scene.push_box((0.12, 6.25, 1.07), (0.06, 0.5, 0.06), materials::wood(balcony_wood));
        scene.push_box((1.48, 6.25, 1.07), (0.06, 0.5, 0.06), materials::wood(balcony_wood));
        scene.push_box((0.2, 5.75, 0.8), (0.08, 0.3, 0.4), materials::wood(balcony_wood));
        scene.push_box((1.4, 5.75, 0.8), (0.08, 0.3, 0.4), materials::wood(balcony_wood));

        // 4.5 Árbol (5 cubos). Sin rotar TR03/TR05 (opcional en las
        // instrucciones, y este raytracer solo soporta cajas alineadas a
        // los ejes).
        scene.push_box((2.8, 4.6, 0.0), (0.3, 1.2, 0.3), materials::wood(Color::from_hex(0x6B4226)));
        scene.push_box((2.8, 5.5, 0.0), (1.5, 0.7, 1.5), materials::sand(Color::from_hex(0x5E8A3C)));
        scene.push_box((2.8, 6.15, 0.0), (1.1, 0.7, 1.1), materials::sand(Color::from_hex(0x6B9A44)));
        scene.push_box((2.8, 6.75, 0.0), (0.7, 0.6, 0.7), materials::sand(Color::from_hex(0x7DAA4D)));
        scene.push_box((2.8, 7.2, 0.0), (0.35, 0.4, 0.35), materials::sand(Color::from_hex(0x8DB85A)));

        // 4.6 Detalles (4 cubos): jardinera y arbustos.
        scene.push_box((1.4, 4.2, 0.85), (1.0, 0.4, 0.4), materials::wood(Color::from_hex(0x8A4B35)));
        scene.push_box((-2.7, 4.2, 1.4), (0.9, 0.4, 0.7), materials::sand(Color::from_hex(0x4F7A33)));
        scene.push_box((-3.2, 4.15, 0.7), (0.5, 0.3, 0.5), materials::sand(Color::from_hex(0x4F7A33)));
        scene.push_box((2.0, 4.15, 1.4), (0.6, 0.3, 0.5), materials::sand(Color::from_hex(0x5E8A3C)));

        scene
    }

    /// 4.7 Humo animado (6 cubos): suben desde la chimenea, ondulan y se
    /// reciclan en loop. Se reconstruye en cada frame con el tiempo actual
    /// en vez de vivir en la escena estática, porque sus posiciones cambian
    /// todo el tiempo.
    pub fn smoke_cubes(time: f32) -> Vec<Box<dyn RayIntersect>> {
        let smoke = materials::sand(Color::from_hex(0xF5EBDD));
        let mut puffs: Vec<Box<dyn RayIntersect>> = Vec::with_capacity(6);

        for i in 0..6 {
            let t = (time * 0.25 + i as f32 / 6.0).fract();
            let y = 9.8 + t * 2.6;
            let x = 1.4 + 0.25 * (time * 1.5 + i as f32 * 1.2 + y * 2.0).sin();
            let z = -1.0;
            let size = 0.2 + 0.25 * (PI * t).sin();

            puffs.push(Box::new(Cube::new(
                Vec3::new(x, y, z),
                Vec3::new(size, size, size),
                smoke,
            )));
        }

        puffs
    }

    fn push_box(&mut self, center: (f32, f32, f32), size: (f32, f32, f32), material: Material) {
        self.objects.push(Box::new(Cube::new(
            Vec3::new(center.0, center.1, center.2),
            Vec3::new(size.0, size.1, size.2),
            material,
        )));
    }
}
