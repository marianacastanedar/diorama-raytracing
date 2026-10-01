use crate::cube::Cube;
use crate::materials;
use crate::ray_intersect::{Material, RayIntersect};
use nalgebra_glm::Vec3;

pub struct Scene {
    pub objects: Vec<Box<dyn RayIntersect>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { objects: Vec::new() }
    }

    /// Isla minimalista: base de arena, acantilado de roca, muelle de madera
    /// y un par de detalles en vidrio. ~26 cubos en total.
    pub fn create_island_scene() -> Self {
        let sand = materials::sand();
        let rock = materials::rock();
        let wood = materials::wood();
        let glass = materials::glass();

        let mut scene = Scene::new();

        // Base de arena: 4x3 cubos a nivel del agua.
        for x in -1..3 {
            for z in -1..2 {
                scene.push_cube(Vec3::new(x as f32, -0.5, z as f32), 1.0, sand);
            }
        }

        // Acantilado rocoso en la esquina de la isla: base 2x2 con una torre.
        for x in 2..4 {
            for z in -2..0 {
                scene.push_cube(Vec3::new(x as f32, 0.5, z as f32), 1.0, rock);
            }
        }
        scene.push_cube(Vec3::new(2.0, 1.5, -1.0), 1.0, rock);
        scene.push_cube(Vec3::new(3.0, 1.5, -1.0), 1.0, rock);
        scene.push_cube(Vec3::new(2.0, 2.5, -1.0), 1.0, rock);

        // Muelle de madera que se adentra en el agua, con un poste elevado.
        for x in -5..-1 {
            scene.push_cube(Vec3::new(x as f32, -0.5, 0.0), 1.0, wood);
        }
        scene.push_cube(Vec3::new(-4.0, 0.5, 0.0), 1.0, wood);

        // Boyas de vidrio junto al muelle.
        scene.push_cube(Vec3::new(-3.0, 0.0, 1.2), 0.4, glass);
        scene.push_cube(Vec3::new(-2.0, 0.0, -1.2), 0.4, glass);

        scene.add_house(Vec3::new(-0.5, 0.0, 0.5));

        scene
    }

    /// Casita voxel de dos pisos sobre la arena: paredes, techo con alero,
    /// chimenea, puerta, un par de ventanas y un escalón de entrada.
    /// `center` es el punto medio del footprint (2x2) a nivel del piso (y=0).
    fn add_house(&mut self, center: Vec3) {
        let wall = materials::wall();
        let roof = materials::roof();
        let door = materials::wood();
        let window = materials::glass();
        let chimney = materials::rock();

        let (cx, cz) = (center.x, center.z);

        // Paredes: footprint 2x2, dos pisos.
        for dx in [-0.5, 0.5] {
            for dz in [-0.5, 0.5] {
                for floor in [0.5, 1.5] {
                    self.push_cube(Vec3::new(cx + dx, floor, cz + dz), 1.0, wall);
                }
            }
        }

        // Techo: una losa con alero (más grande que el footprint) y remate.
        self.push_cube(Vec3::new(cx, 3.1, cz), 2.2, roof);
        self.push_cube(Vec3::new(cx + 0.65, 4.3, cz + 0.15), 0.35, chimney);

        // Puerta al frente (+Z) y ventanas en el piso superior. La cara
        // exterior de la pared está en cz+1: puerta/ventana se apoyan justo
        // ahí hacia afuera, sin enterrarse en la pared (si se solapan da
        // z-fighting: caras casi coplanares parpadean).
        let wall_face = cz + 1.0;
        self.push_cube(Vec3::new(cx, 0.6, wall_face + 0.25), 0.5, door);
        self.push_cube(Vec3::new(cx - 0.5, 1.5, wall_face + 0.15), 0.3, window);
        self.push_cube(Vec3::new(cx + 0.5, 1.5, wall_face + 0.15), 0.3, window);
    }

    fn push_cube(&mut self, center: Vec3, size: f32, material: Material) {
        self.objects.push(Box::new(Cube::new(center, size, material)));
    }
}
