mod aarects;
pub mod bhv;
pub mod camera;
pub mod hittable;
pub mod image_texture;
pub mod materials;
pub mod raytrace;
pub mod rngator;
pub mod shapes;
pub mod textures;
pub mod transforms;
pub mod vec;
pub mod volumes;
pub mod worlds;

use camera::Camera;
use clap::{App, Arg, ArgMatches};
use raytrace::{RayTracer, RecursiveRayTracer};
use rngator::Rngator;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use vec::{Point3, Vec3};

struct Parameters {
    pub world: Box<dyn worlds::World>,
    pub seed: Option<u64>,
    pub randomized_rendering: bool,

    pub aspect_ratio: f64,
    pub render: raytrace::RenderingParams,
    pub max_depth: i32,

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

fn undef_arg<'a>(name: &'a str, help: &'a str) -> Arg<'a, 'a> {
    Arg::with_name(name).long(name).help(help).takes_value(true)
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
    let mut worlds = worlds::worlds();
    let world_names: Vec<&'static str> = worlds.iter().map(|w| w.name()).collect();
    let matches = App::new("mulambda raytracer")
        .version("0.1")
        .arg(arg("aspect_ratio", "16:9"))
        .arg(arg("image_width", "400"))
        .arg(arg("samples_per_pixel", "200"))
        .arg(arg("max_depth", "50"))
        .arg(undef_arg("lookfrom", "[point] camera position"))
        .arg(undef_arg("lookat", "[point] point that camera looks at"))
        .arg(arg("up", "0,1.0,0"))
        .arg(undef_arg("field_of_view", "[float] field of view, in degrees"))
        .arg(arg("aperture", "0.0"))
        .arg(Arg::with_name("focus_dist").long("focus_dist").takes_value(true))
        .arg(
            Arg::with_name("world")
                .long("world")
                .takes_value(true)
                .possible_values(&world_names)
                .default_value("simple"),
        )
        .arg(Arg::with_name("seed").long("seed").takes_value(true))
        .arg(Arg::with_name("randomized_rendering").long("randomized_rendering").short("rr"))
        .get_matches();

    fn val<'a, T>(m: &ArgMatches<'a>, name: &str) -> T
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        m.value_of(name).unwrap().parse::<T>().unwrap()
    }

    let world_name = matches.value_of("world").unwrap();
    let world = worlds.remove(worlds.iter().position(|w| w.name() == world_name).unwrap());

    let aspect_ratio = parse_aspect_ratio(matches.value_of("aspect_ratio").unwrap());
    let image_width = val::<usize>(&matches, "image_width");

    let lookfrom = matches.value_of("lookfrom").map_or(world.camera().lookfrom, |v| parse_vector(&v));
    let lookat = matches.value_of("lookat").map_or(world.camera().lookat, |v| parse_vector(&v));
    let field_of_view =
        matches.value_of("field_of_view").map_or(world.camera().field_of_view, |v| v.parse::<f64>().unwrap());

    let focus_dist = match matches.value_of("focus_dist") {
        None => (lookat - lookfrom).length(),
        Some(v) => v.parse::<f64>().unwrap(),
    };

    Parameters {
        world,
        seed: matches.value_of("seed").map(|v| v.parse::<u64>().unwrap()),
        randomized_rendering: matches.is_present("randomized_rendering"),
        aspect_ratio,
        render: raytrace::RenderingParams {
            image_width,
            image_height: (image_width as f64 / aspect_ratio) as usize,
            samples_per_pixel: val::<i32>(&matches, "samples_per_pixel"),
        },
        max_depth: val::<i32>(&matches, "max_depth"),
        lookfrom,
        lookat,
        up: parse_vector(matches.value_of("up").unwrap()),
        field_of_view,
        aperture: val::<f64>(&matches, "aperture"),
        focus_dist,
    }
}

fn do_tracing<T>(
    params: Parameters,
    camera: &Camera,
    world: &dyn hittable::Hittable,
    background: &dyn raytrace::Background,
    rngator: T,
) where
    T: Rngator,
{
    // Render
    println!("P3\n{} {}\n255", params.render.image_width, params.render.image_height);
    let start_time = Instant::now();
    let remaining_count = AtomicUsize::new(usize::MAX);
    let rt = RayTracer::new_with_rng(
        camera,
        world,
        background,
        params.render,
        RecursiveRayTracer { max_depth: params.max_depth },
        rngator,
    );
    let last_logged = AtomicUsize::new(0);
    let image = rt.render(|_, total| {
        const R: Ordering = Ordering::Relaxed;
        let _ = remaining_count.compare_exchange(usize::MAX, total, R, R);
        let remaining = remaining_count.fetch_sub(1, R) - 1;
        if remaining == 0 {
            eprint!("\r{:50}", "Done!");
            return;
        }
        let elapsed = start_time.elapsed().as_millis() as usize;
        let ll = last_logged.load(R);
        if ll < elapsed && elapsed - ll > 300 {
            match last_logged.compare_exchange_weak(ll, elapsed, R, R) {
                Err(_) => return, // Someone got to print first, exiting.
                Ok(_) => eprint!("\rRemaining: {:3}%  ", remaining * 100 / total),
            }
        }
    });
    eprintln!("\nRendered in {:.3}s", start_time.elapsed().as_secs_f32());
    for line in image.iter().rev() {
        for (r, g, b) in line.iter() {
            println!("{} {} {}", r, g, b);
        }
    }
}
fn do_it<T>(parameters: Parameters, rngator: T)
where
    T: Rngator,
{
    let mut rng = rngator.rng(0);

    // World
    let world = parameters.world.build(&mut rng);
    let background = parameters.world.background();

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

    if parameters.randomized_rendering {
        do_tracing(parameters, &cam, world.as_ref(), background.as_ref(), rngator::ThreadRngator {});
    } else {
        do_tracing(parameters, &cam, world.as_ref(), background.as_ref(), rngator);
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
