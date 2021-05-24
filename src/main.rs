pub mod bhv;
pub mod camera;
pub mod hittable;
pub mod materials;
pub mod raytrace;
pub mod rngator;
pub mod shapes;
pub mod vec;

use camera::Camera;
use clap::{App, Arg};
use hittable::Hittable;
use materials::{Dielectric, Lambertian, Metal};
use rand::Rng;
use raytrace::RayTracer;
use rngator::Rngator;
use shapes::Sphere;
use std::sync::atomic::{self, AtomicIsize, AtomicUsize};
use std::time::Instant;
use vec::{Color, Point3, Vec3};

struct Parameters {
    pub random_world: bool,
    pub seed: Option<u64>,

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
        .arg(Arg::with_name("seed").long("seed").takes_value(true))
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
        seed: matches.value_of("seed").map(|v| v.parse::<u64>().unwrap()),
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

fn rnd01(rng: &mut dyn rand::RngCore) -> f64 {
    rng.gen_range(0.0..1.0)
}

fn random_world<'a>(rng: &mut dyn rand::RngCore) -> Box<dyn Hittable + 'a> {
    let mut world = bhv::SceneBuilder::new();

    let ground_material = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rnd01(rng);
            let center = Point3::new(a as f64 + 0.9 * rnd01(rng), 0.2, b as f64 + 0.9 * rnd01(rng));

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::random_unit(rng) * Color::random_unit(rng);
                    world.add(Sphere::new(center, 0.2, Lambertian::new(albedo)));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random(0.5, 1.0, rng);
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

fn do_it<T>(parameters: Parameters, rngator: T)
where
    T: Rngator,
{
    let mut rng_box = rngator.rng();
    let rng = rng_box.as_mut();

    // World
    let world: Box<dyn Hittable> =
        if parameters.random_world { random_world(rng) } else { simple_world() };

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
    let remaining_count = AtomicIsize::new(-1);
    let rt = RayTracer::new_with_rng(&cam, world.as_ref(), parameters.render, rngator);
    let last_logged = AtomicUsize::new(0);
    let image = rt.render(|_, total| {
        let _ = remaining_count.compare_exchange(
            -1,
            total as isize,
            atomic::Ordering::Relaxed,
            atomic::Ordering::Relaxed,
        );
        let rem_lines = remaining_count.fetch_sub(1, atomic::Ordering::Relaxed) - 1;
        if rem_lines == 0 {
            eprint!("\r{:50}", "Done!");
            return;
        }
        let elapsed = start_time.elapsed().as_millis() as usize;
        let ll = last_logged.load(atomic::Ordering::Relaxed);
        if ll < elapsed && elapsed - ll > 500 {
            match last_logged.compare_exchange_weak(
                ll,
                elapsed,
                atomic::Ordering::Relaxed,
                atomic::Ordering::Relaxed,
            ) {
                Err(_) => return,
                Ok(_) => eprint!("\rScanlines remaining: {:10}  ", rem_lines),
            }
        }
    });
    eprintln!("\nRendered in {:.3}s", start_time.elapsed().as_secs_f32());
    for j in (0..parameters.render.image_height).rev() {
        for (r, g, b) in image[j].iter() {
            println!("{} {} {}", r, g, b);
        }
    }
}

fn main() {
    // Image
    let parameters = args();
    match parameters.seed {
        None => do_it(parameters, rngator::ThreadRngator {}),
        Some(seed) => do_it(parameters, rngator::SeedableRngator::new(seed)),
    }
}
