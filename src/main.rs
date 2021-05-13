use std::io::{self, Write};
mod datatypes;
mod hittable;

use datatypes::{dot, unit_vector, write_color, Color, Point3, Ray, Vec3};
use hittable::{Hittable, HittableList, Sphere};

fn hit_sphere(center: &Point3, radius: f64, r: &Ray) -> f64 {
    // let C be the shpere center
    // ray r is P(t) = O + t*b, where O is r.orig and b is r.dir
    // Sphere is  (P(t) - C)^2 = radius^2
    // t^2 * b*b + 2tb * (O - C) + (O - C)^2 - radius^2 = 0
    let oc = &r.orig - center;
    let a = r.dir.length_squared();
    let half_b = dot(oc, r.dir);
    let c = oc.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return -1.0;
    } else {
        return (-half_b - discriminant.sqrt()) / a;
    }
}

fn ray_color(ray: &Ray, world: &dyn Hittable) -> Color {
    let white: Color = Color::new(1.0f64, 1.0f64, 1.0f64);
    let blueish: Color = Color::new(0.5f64, 0.7f64, 1.0f64);

    let sphere_center = Point3::new(0.0, 0.0, -1.0);
    let sphere_radius = 0.5;

    match world.hit(ray, 0.0, f64::INFINITY) {
        Some(h) => return 0.5 * (h.normal + Color::new(1.0, 1.0, 1.0)),
        None => {}
    }
    let unit_direction = unit_vector(&ray.dir);
    let t = 0.5f64 * (unit_direction.y() + 1.0f64);
    return (1.0 - t) * white + t * blueish;
}

fn main() {
    // Image
    let aspect_ratio = 16.0f64 / 9.0f64;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as i32;

    // World
    let mut world = HittableList::new();
    world.push(Box::new(Sphere::new(&Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.push(Box::new(Sphere::new(
        &Point3::new(0.0, -100.5, -1.0),
        100.0,
    )));

    // Camera
    let viewport_height = 2.0f64;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0f64;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    // Render

    println!("P3\n{} {}\n255", image_width, image_height);
    for j in (0..image_height).rev() {
        eprint!("\rScanlines remaining: {}", j);
        io::stderr().flush().unwrap();

        for i in 0..image_width {
            let u = (i as f64) / (image_width as f64 - 1.0);
            let v = (j as f64) / (image_height as f64 - 1.0);
            let r = Ray {
                orig: origin,
                dir: lower_left_corner + u * horizontal + v * vertical - origin,
            };
            let c = ray_color(&r, &world);
            write_color(&c, &mut std::io::stdout());
        }
    }
}
