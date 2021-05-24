pub mod bhv;
pub mod camera;
pub mod hittable;
pub mod materials;
pub mod raytrace;
pub mod rngator;
pub mod shapes;
pub mod textures;
pub mod vec;
pub mod worlds;

use camera::Camera;
use clap::{App, Arg, ArgMatches};
use hittable::Hittable;
use raytrace::RayTracer;
use rngator::Rngator;
use std::sync::atomic::{self, AtomicIsize, AtomicUsize};
use std::time::Instant;
use vec::{Point3, Vec3};

struct Parameters {
    pub world: worlds::World,
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
        .arg(
            Arg::with_name("world")
                .long("world")
                .takes_value(true)
                .possible_values(&["simple", "random", "random_chk", "two_spheres"])
                .default_value("simple"),
        )
        .arg(Arg::with_name("seed").long("seed").takes_value(true))
        .get_matches();

    fn val<'a, T>(m: &ArgMatches<'a>, name: &str) -> T
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        m.value_of(name).unwrap().parse::<T>().unwrap()
    }

    let aspect_ratio = parse_aspect_ratio(matches.value_of("aspect_ratio").unwrap());
    let image_width = val::<usize>(&matches, "image_width");
    let lookfrom = parse_vector(&matches.value_of("lookfrom").unwrap());
    let lookat = parse_vector(matches.value_of("lookat").unwrap());
    let focus_dist = match matches.value_of("focus_dist") {
        None => (lookat - lookfrom).length(),
        Some(v) => v.parse::<f64>().unwrap(),
    };

    Parameters {
        world: match matches.value_of("world").unwrap() {
            "simple" => worlds::World::Simple,
            "random" => worlds::World::Random,
            "random_chk" => worlds::World::RandomChk,
            "two_spheres" => worlds::World::TwoSpheres,
            _ => panic!(),
        },
        seed: matches.value_of("seed").map(|v| v.parse::<u64>().unwrap()),
        aspect_ratio,
        render: raytrace::RenderingParams {
            image_width,
            image_height: (image_width as f64 / aspect_ratio) as usize,
            samples_per_pixel: val::<i32>(&matches, "samples_per_pixel"),
            max_depth: val::<i32>(&matches, "max_depth"),
        },
        lookfrom,
        lookat,
        up: parse_vector(matches.value_of("up").unwrap()),
        field_of_view: val::<f64>(&matches, "field_of_view"),
        aperture: val::<f64>(&matches, "aperture"),
        focus_dist,
    }
}

fn do_it<T>(parameters: Parameters, rngator: T)
where
    T: Rngator,
{
    let mut rng_box = rngator.rng();
    let rng = rng_box.as_mut();

    // World
    let world: Box<dyn Hittable> = match parameters.world {
        worlds::World::Simple => worlds::simple_world(rng),
        worlds::World::Random => worlds::random_world(rng),
        worlds::World::RandomChk => worlds::random_world_chk(rng),
        worlds::World::TwoSpheres => worlds::two_spheres(),
    };

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
