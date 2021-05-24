mod bhv;
mod camera;
mod hittable;
mod materials;
mod raytrace;
mod shapes;
mod vec;

use camera::Camera;
use clap::{App, Arg};
use hittable::Hittable;
use materials::{Dielectric, Lambertian, Metal};
use rand::Rng;
use raytrace::RayTracer;
use shapes::Sphere;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use vec::{unit_vector, Color, Point3, Ray, Vec3};

struct Parameters {
    pub random_world: bool,

    pub aspect_ratio: f64,
    pub render: raytrace::RenderingParams,

    pub lookfrom: Point3,
    pub lookat: Point3,
    pub up: Vec3,
    pub field_of_view: f64, // degrees, (0..180)
    pub aperture: f64,
    pub focus_dist: f64,
}

fn arg<'a>(name: &'a str, default_value: &'a str) -> Arg<'a, 'a> {
    Arg::with_name(name).long(name).takes_value(true).default_value(default_value)
}

fn parse_aspect_ratio(s: &str) -> f64 {
    let v: Vec<&str> = s.split(':').collect();
    return v[0].parse::<i32>().unwrap() as f64 / v[1].parse::<i32>().unwrap() as f64;
}

fn parse_vector(s: &str) -> Vec3 {
    let input: Vec<&str> = s.split(',').collect();
    let mut e = [0.0, 0.0, 0.0];
    for i in 0..3 {
        e[i] = input[i].parse::<f64>().unwrap();
    }

    Vec3 { e }
}

fn args() -> Parameters {
    let matches = App::new("mulambda raytracer")
        .version("0.1")
        .arg(arg("aspect_ratio", "16:9"))
        .arg(arg("image_width", "400"))
        .arg(arg("samples_per_pixel", "200"))
        .arg(arg("max_depth", "50"))
        .arg(arg("lookfrom", "-2,2,1"))
        .arg(arg("lookat", "0,0,-1"))
        .arg(arg("up", "0,1.0,0"))
        .arg(arg("field_of_view", "90.0"))
        .arg(arg("aperture", "0.0"))
        .arg(Arg::with_name("focus_dist").long("focus_dist").takes_value(true))
        .arg(Arg::with_name("random_world").long("random_world"))
        .get_matches();
    let aspect_ratio = parse_aspect_ratio(matches.value_of("aspect_ratio").unwrap());
    let image_width = matches.value_of("image_width").unwrap().parse::<usize>().unwrap();
    let lookfrom = parse_vector(matches.value_of("lookfrom").unwrap());
    let lookat = parse_vector(matches.value_of("lookat").unwrap());
    let focus_dist = match matches.value_of("focus_dist") {
        None => (lookat - lookfrom).length(),
        Some(v) => v.parse::<f64>().unwrap(),
    };

    Parameters {
        random_world: matches.is_present("random_world"),
        aspect_ratio,
        render: raytrace::RenderingParams {
            image_width,
            image_height: (image_width as f64 / aspect_ratio) as usize,
            samples_per_pixel: matches
                .value_of("samples_per_pixel")
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            max_depth: matches.value_of("max_depth").unwrap().parse::<i32>().unwrap(),
        },
        lookfrom,
        lookat,
        up: parse_vector(matches.value_of("up").unwrap()),
        field_of_view: matches.value_of("field_of_view").unwrap().parse::<f64>().unwrap(),
        aperture: matches.value_of("aperture").unwrap().parse::<f64>().unwrap(),
        focus_dist,
    }
}

fn simple_world<'a>() -> Box<dyn Hittable + 'a> {
    let mat_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let mat_center = Lambertian::new(Color::new(0.1, 0.3, 0.5));
    let mat_left = Dielectric::new(1.5);
    let mat_right = Metal::new(Color::new(0.8, 0.6, 0.2), 0.0);

    let mut world = bhv::SceneBuilder::new();

    world
        .add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground))
        .add(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center))
        .add(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left.clone()))
        .add(Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.4, mat_left))
        .add(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right));

    let bhv = bhv::BHV::new(&mut world);
    Box::new(bhv)
}

fn random_world<'a>() -> Box<dyn Hittable + 'a> {
    let mut world = bhv::SceneBuilder::new();

    let ground_material = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material));

    fn rnd() -> f64 {
        rand::thread_rng().gen_range(0.0..1.0)
    }

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rnd();
            let center = Point3::new(a as f64 + 0.9 * rnd(), 0.2, b as f64 + 0.9 * rnd());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::random_unit() * Color::random_unit();
                    world.add(Sphere::new(center, 0.2, Lambertian::new(albedo)));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.0);
                    let fuzz = rand::thread_rng().gen_range(0.0..0.5);
                    world.add(Sphere::new(center, 0.2, Metal::new(albedo, fuzz)));
                } else {
                    world.add(Sphere::new(center, 0.2, Dielectric::new(1.5)));
                }
            }
        }
    }

    world
        .add(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Dielectric::new(1.5)))
        .add(Sphere::new(
            Point3::new(-4.0, 1.0, 0.0),
            1.0,
            Lambertian::new(Color::new(0.4, 0.2, 0.1)),
        ))
        .add(Sphere::new(
            Point3::new(4.0, 1.0, 0.0),
            1.0,
            Metal::new(Color::new(0.7, 0.6, 0.5), 0.0),
        ));

    Box::new(bhv::BHV::new(&mut world))
}

fn main() {
    let mut rng = rand::thread_rng();
    // Image
    let parameters = args();

    // World
    let world: Box<dyn Hittable> =
        if parameters.random_world { random_world() } else { simple_world() };

    // Camera
    let cam = Camera::new(
        parameters.lookfrom,
        parameters.lookat,
        parameters.up,
        parameters.field_of_view,
        parameters.aspect_ratio,
        parameters.aperture,
        parameters.focus_dist,
    );
    //let cam = Camera::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0),
    //    parameters.field_of_view, aspect_ratio);

    // Render
    println!("P3\n{} {}\n255", parameters.render.image_width, parameters.render.image_height);
    let start_time = Instant::now();

    let rt = RayTracer::new(&cam, world.as_ref(), parameters.render);
    let image = rt.render(|j| {
        eprint!("\rScanlines remaining: {}", parameters.render.image_height - j);
    });
    eprintln!("\nRendered in {:.3}s", start_time.elapsed().as_secs_f32());
    for j in (0..parameters.render.image_height).rev() {
        for (r, g, b) in image[j].iter() {
            println!("{} {} {}", r, g, b);
        }
    }
}
