mod camera;
mod datatypes;
mod hittable;
mod materials;

use camera::Camera;
use datatypes::{unit_vector, write_color, Color, Point3, Ray, Vec3};
use hittable::{Hittable, HittableList};
use materials::{Lambertian, Material, Metal};
use rand::Rng;
use std::io::{self, Write};
use std::rc::Rc;

fn ray_color(ray: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    if depth <= 0 {
        return Color::ZERO;
    }
    match world.hit(ray, 0.001, f64::INFINITY) {
        Some(h) => match h.material.scatter(ray, &h) {
            Some((attenuation, scattered)) => {
                return attenuation * ray_color(&scattered, world, depth - 1);
            }
            None => {
                return Color::ZERO;
            }
        },
        None => {
            let white: Color = Color::new(1.0f64, 1.0f64, 1.0f64);
            let blueish: Color = Color::new(0.5f64, 0.7f64, 1.0f64);
            let unit_direction = unit_vector(&ray.dir);
            let t = 0.5f64 * (unit_direction.y() + 1.0f64);
            return (1.0 - t) * white + t * blueish;
        }
    }
}

fn main() {
    // Image
    let aspect_ratio = 16.0f64 / 9.0f64;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let samples_per_pixel = 100;
    let max_depth = 50;

    // World
    let mut world = HittableList::new();

    let mat_ground: Rc<dyn Material> = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_center: Rc<dyn Material> = Rc::new(Lambertian::new(Color::new(0.7, 0.3, 0.3)));
    let mat_left: Rc<dyn Material> = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.8)));
    let mat_right: Rc<dyn Material> = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2)));

    world.push_sphere(Vec3::new(0.0, -100.5, -1.0), 100.0, &mat_ground);
    world.push_sphere(Vec3::new(0.0, 0.0, -1.0), 0.5, &mat_center);
    world.push_sphere(Vec3::new(-1.0, 0.0, -1.0), 0.5, &mat_left);
    world.push_sphere(Vec3::new(1.0, 0.0, -1.0), 0.5, &mat_right);

    // Camera
    let cam = Camera::new();

    // Render
    let mut rng = rand::thread_rng();

    println!("P3\n{} {}\n255", image_width, image_height);
    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {}    ", j);
        io::stderr().flush().unwrap();

        for i in 0..image_width {
            let mut pixel_color = Color::ZERO;
            for _ in 0..samples_per_pixel {
                let u = ((i as f64) + rng.gen_range(0.0..1.0)) / (image_width as f64 - 1.0);
                let v = ((j as f64) + rng.gen_range(0.0..1.0)) / (image_height as f64 - 1.0);
                let r = cam.get_ray(u, v);
                pixel_color = pixel_color + ray_color(&r, &world, max_depth);
            }
            write_color(&pixel_color, samples_per_pixel, &mut std::io::stdout());
        }
    }
}
