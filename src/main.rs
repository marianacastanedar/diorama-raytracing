mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod plane;
mod ray_intersect;
mod raytracer;

use camera::Camera;
use color::Color;
use cube::Cube;
use framebuffer::Framebuffer;
use light::Light;
use nalgebra_glm::Vec3;
use plane::Plane;
use ray_intersect::{Material, RayIntersect};
use raylib::prelude::RaylibDraw;
use raytracer::cast_ray;
use std::f32::consts::PI;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const FOV: f32 = PI / 3.0;

fn build_scene() -> Vec<Box<dyn RayIntersect>> {
    let sand = Material::new(Color::new(217, 191, 140), 0.9);
    let rock = Material::new(Color::new(120, 120, 125), 0.6);
    let water = Material::new(Color::new(40, 110, 160), 0.7);

    vec![
        Box::new(Cube::new(Vec3::new(0.0, -0.5, 0.0), 1.0, sand)),
        Box::new(Cube::new(Vec3::new(1.0, -0.5, 0.0), 1.0, sand)),
        Box::new(Cube::new(Vec3::new(0.5, 0.5, -0.5), 1.0, rock)),
        Box::new(Plane::new(Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0), water)),
    ]
}

fn render(framebuffer: &mut Framebuffer, camera: &Camera, objects: &[Box<dyn RayIntersect>], light: &Light) {
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let tan_fov = (FOV / 2.0).tan();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_x = (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect_ratio * tan_fov;
            let screen_y = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * tan_fov;

            let ray_direction = camera.basis_change(&Vec3::new(screen_x, screen_y, -1.0));
            let color = cast_ray(&camera.eye, &ray_direction, objects, light);

            framebuffer.set_current_color(color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Diorama Marino - Raytracing")
        .build();

    rl.set_target_fps(60);

    let camera = Camera::new(
        Vec3::new(0.0, 1.5, 4.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let light = Light::new(Vec3::new(3.0, 5.0, 3.0), Color::new(255, 255, 255), 1.0);
    let objects = build_scene();

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    render(&mut framebuffer, &camera, &objects, &light);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(raylib::color::Color::BLACK);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let hex = framebuffer.buffer[y * WIDTH + x];
                let r = ((hex >> 16) & 0xFF) as u8;
                let g = ((hex >> 8) & 0xFF) as u8;
                let b = (hex & 0xFF) as u8;
                d.draw_pixel(x as i32, y as i32, raylib::color::Color::new(r, g, b, 255));
            }
        }
    }
}
