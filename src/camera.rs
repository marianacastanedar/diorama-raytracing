use nalgebra_glm::Vec3;
use std::f32::consts::PI;

// Asimétrico a propósito: mirando desde arriba (pitch negativo) puede subir
// casi hasta vertical, pero bajando (pitch positivo acerca la cámara al
// nivel del agua en y=0) se frena bastante antes para no meter la cámara
// bajo el agua — con el target en y=5.0 y radio hasta MAX_RADIUS, 6° de
// margen deja el ojo de la cámara por encima del agua con margen.
const PITCH_LIMIT_UP: f32 = PI / 2.0 - 0.1;
const PITCH_LIMIT_DOWN: f32 = 6.0 * PI / 180.0;
const MIN_RADIUS: f32 = 10.0;
const MAX_RADIUS: f32 = 30.0;

#[derive(Clone, Copy)]
pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        Camera { eye, center, up }
    }

    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        let forward = (self.center - self.eye).normalize();
        let right = forward.cross(&self.up).normalize();

        let up = right.cross(&forward).normalize();

        let rotated = vector.x * right + vector.y * up - vector.z * forward;

        rotated.normalize()
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let radius_vector = self.eye - self.center;
        let radius = radius_vector.magnitude();

        let current_yaw = radius_vector.z.atan2(radius_vector.x);
        let radius_xz =
            (radius_vector.x * radius_vector.x + radius_vector.z * radius_vector.z).sqrt();
        let current_pitch = (-radius_vector.y).atan2(radius_xz);

        let new_yaw = (current_yaw + delta_yaw) % (2.0 * PI);
        let new_pitch = (current_pitch + delta_pitch).clamp(-PITCH_LIMIT_UP, PITCH_LIMIT_DOWN);

        self.eye = self.center
            + Vec3::new(
                radius * new_yaw.cos() * new_pitch.cos(),
                -radius * new_pitch.sin(),
                radius * new_yaw.sin() * new_pitch.cos(),
            );
    }

    pub fn zoom(&mut self, delta: f32) {
        let radius_vector = self.eye - self.center;
        let direction = radius_vector.normalize();
        let radius = (radius_vector.magnitude() + delta).clamp(MIN_RADIUS, MAX_RADIUS);

        self.eye = self.center + direction * radius;
    }
}
