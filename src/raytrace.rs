use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::rngator;
use crate::vec::{unit_vector, Color, Ray};
use rand::Rng;
use rayon::prelude::*;

pub fn ray_color(
    ray: &Ray,
    world: &dyn Hittable,
    depth: i32,
    rng: &mut dyn rand::RngCore,
) -> Color {
    if depth <= 0 {
        return Color::ZERO;
    }
    match world.hit(ray, 0.001, f64::INFINITY) {
        Some(h) => match h.material.scatter(ray, &h, rng) {
            Some((attenuation, scattered)) => {
                return attenuation * ray_color(&scattered, world, depth - 1, rng);
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

#[derive(Copy, Clone)]
pub struct RenderingParams {
    pub samples_per_pixel: i32,
    pub image_height: usize,
    pub image_width: usize,
    pub max_depth: i32,
}

pub type RGB = (i32, i32, i32);

pub fn to_rgb(color: &Color, samples_per_pixel: i32) -> RGB {
    let scale = 1.0f64 / samples_per_pixel as f64;
    let r = (color.r() * scale).sqrt();
    let g = (color.g() * scale).sqrt();
    let b = (color.b() * scale).sqrt();
    let ir = (255.999f64 * r.clamp(0.0, 0.99999999)) as i32;
    let ig = (255.999f64 * g.clamp(0.0, 0.99999999)) as i32;
    let ib = (255.999f64 * b.clamp(0.0, 0.99999999)) as i32;
    (ir, ig, ib)
}

pub struct RayTracer<'a, T = rngator::ThreadRngator>
where
    T: rngator::Rngator,
{
    camera: &'a Camera,
    world: &'a dyn Hittable,
    parameters: RenderingParams,
    rng: T,
}

impl<'a> RayTracer<'a> {
    pub fn new(
        camera: &'a Camera,
        world: &'a dyn Hittable,
        parameters: RenderingParams,
    ) -> RayTracer<'a, rngator::ThreadRngator> {
        RayTracer::new_with_rng(camera, world, parameters, rngator::ThreadRngator {})
    }
}

impl<'a, T: rngator::Rngator> RayTracer<'a, T> {
    pub fn new_with_rng(
        camera: &'a Camera,
        world: &'a dyn Hittable,
        parameters: RenderingParams,
        rng: T,
    ) -> RayTracer<'a, T> {
        RayTracer { camera, world, parameters, rng }
    }

    pub fn render_line(&self, j: usize, result: &mut [RGB], rng: &mut dyn rand::RngCore) {
        if result.len() != self.parameters.image_width {
            panic!()
        }

        for i in 0..self.parameters.image_width {
            result[i] = self.render_pixel(i, j, rng)
        }
    }

    pub fn render<Logger>(&self, logger: Logger) -> Vec<Vec<RGB>>
    where
        Logger: Fn(usize, usize) -> () + Sync,
    {
        (0..self.parameters.image_height)
            .into_par_iter()
            .map(|j| {
                let mut rng_box = self.rng.rng();
                let rng = rng_box.as_mut();
                let mut line = vec![(0, 0, 0); self.parameters.image_width];
                self.render_line(j, line.as_mut_slice(), rng);
                logger(j, self.parameters.image_height);
                line
            })
            .collect()
    }

    pub fn render_pixel(&self, i: usize, j: usize, rng: &mut dyn rand::RngCore) -> RGB {
        let mut pixel_color = Color::ZERO;
        for _ in 0..self.parameters.samples_per_pixel {
            let u =
                ((i as f64) + rng.gen_range(0.0..1.0)) / (self.parameters.image_width as f64 - 1.0);
            let v = ((j as f64) + rng.gen_range(0.0..1.0))
                / (self.parameters.image_height as f64 - 1.0);
            let r = self.camera.get_ray(u, v, rng);
            pixel_color = pixel_color + ray_color(&r, self.world, self.parameters.max_depth, rng);
        }

        to_rgb(&pixel_color, self.parameters.samples_per_pixel)
    }
}
