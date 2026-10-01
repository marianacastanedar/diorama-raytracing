use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

const EPSILON: f32 = 1e-6;

pub struct Cube {
    pub center: Vec3,
    pub size: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: Vec3, material: Material) -> Self {
        Cube {
            center,
            size,
            material,
        }
    }

    fn min_bound(&self) -> Vec3 {
        self.center - self.size / 2.0
    }

    fn max_bound(&self) -> Vec3 {
        self.center + self.size / 2.0
    }
}

fn axis_unit(axis: usize, sign: f32) -> Vec3 {
    match axis {
        0 => Vec3::new(sign, 0.0, 0.0),
        1 => Vec3::new(0.0, sign, 0.0),
        _ => Vec3::new(0.0, 0.0, sign),
    }
}

fn axis_slab(origin: f32, direction: f32, min: f32, max: f32) -> Option<(f32, f32, f32)> {
    if direction.abs() < EPSILON {
        return if origin < min || origin > max {
            None
        } else {
            Some((f32::NEG_INFINITY, f32::INFINITY, 0.0))
        };
    }

    let t1 = (min - origin) / direction;
    let t2 = (max - origin) / direction;

    if t1 < t2 {
        Some((t1, t2, -1.0))
    } else {
        Some((t2, t1, 1.0))
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let min_bound = self.min_bound();
        let max_bound = self.max_bound();

        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        let mut normal = Vec3::new(0.0, 0.0, 0.0);

        for axis in 0..3 {
            let (near, far, sign) = axis_slab(
                ray_origin[axis],
                ray_direction[axis],
                min_bound[axis],
                max_bound[axis],
            )?;

            if near > t_min {
                t_min = near;
                normal = axis_unit(axis, sign);
            }
            t_max = t_max.min(far);

            if t_min > t_max {
                return None;
            }
        }

        if t_max < EPSILON {
            return None;
        }

        let distance = if t_min > EPSILON { t_min } else { t_max };

        Some(Intersect {
            point: ray_origin + ray_direction * distance,
            normal,
            distance,
            material: self.material,
        })
    }
}
