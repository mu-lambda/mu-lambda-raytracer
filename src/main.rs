mod camera;
mod datatypes;
mod hittable;
mod materials;

use camera::Camera;
use clap::{App, Arg};
use datatypes::{unit_vector, write_color, Color, Ray, Vec3};
use hittable::{Hittable, HittableList};
use materials::{Dielectric, Lambertian, Material, Metal};
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

struct Parameters {
    pub image_width: i32,
    pub image_height: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,

    pub field_of_view: f64, // degrees, (0..180)
}

fn arg<'a>(name: &'a str, default_value: &'a str) -> Arg<'a, 'a> {
    Arg::with_name(name).long(name).takes_value(true).default_value(default_value)
}

fn args(aspect_ratio: f64) -> Parameters {
    let matches = App::new("mulambda raytracer")
        .version("0.1")
        .arg(arg("image_width", "400"))
        .arg(arg("samples_per_pixel", "200"))
        .arg(arg("max_depth", "50"))
        .arg(arg("field_of_view", "90.0"))
        .get_matches();
    let image_width = matches.value_of("image_width").unwrap().parse::<i32>().unwrap();
    Parameters {
        image_width,
        image_height: (image_width as f64 / aspect_ratio) as i32,
        samples_per_pixel: matches.value_of("samples_per_pixel").unwrap().parse::<i32>().unwrap(),
        max_depth: matches.value_of("max_depth").unwrap().parse::<i32>().unwrap(),

        field_of_view: matches.value_of("field_of_view").unwrap().parse::<f64>().unwrap(),
    }
}

fn main() {
    // Image
    let aspect_ratio = 16.0f64 / 9.0f64;
    let parameters = args(aspect_ratio);
    // World
    let mut world = HittableList::new();

    let mat_ground: Rc<dyn Material> = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_center: Rc<dyn Material> = Rc::new(Lambertian::new(Color::new(0.1, 0.3, 0.5)));
    let mat_left: Rc<dyn Material> = Rc::new(Dielectric::new(1.5));
    let mat_right: Rc<dyn Material> = Rc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.push_sphere(Vec3::new(0.0, -100.5, -1.0), 100.0, &mat_ground);
    world.push_sphere(Vec3::new(0.0, 0.0, -1.0), 0.5, &mat_center);
    world.push_sphere(Vec3::new(-1.0, 0.0, -1.0), 0.5, &mat_left);
    world.push_sphere(Vec3::new(-1.0, 0.0, -1.0), -0.4, &mat_left);
    world.push_sphere(Vec3::new(1.0, 0.0, -1.0), 0.5, &mat_right);

    // Camera
    let cam = Camera::new(parameters.field_of_view, aspect_ratio);

    // Render
    let mut rng = rand::thread_rng();

    println!("P3\n{} {}\n255", parameters.image_width, parameters.image_height);
    for j in (0..parameters.image_height).rev() {
        eprint!("\rScanlines remaining: {}    ", j);
        io::stderr().flush().unwrap();

        for i in 0..parameters.image_width {
            let mut pixel_color = Color::ZERO;
            for _ in 0..parameters.samples_per_pixel {
                let u = ((i as f64) + rng.gen_range(0.0..1.0)) / (parameters.image_width as f64 - 1.0);
                let v = ((j as f64) + rng.gen_range(0.0..1.0)) / (parameters.image_height as f64 - 1.0);
                let r = cam.get_ray(u, v);
                pixel_color = pixel_color + ray_color(&r, &world, parameters.max_depth);
            }
            write_color(&pixel_color, parameters.samples_per_pixel, &mut std::io::stdout());
        }
    }
}
