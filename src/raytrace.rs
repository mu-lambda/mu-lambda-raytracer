use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::rngator;
use crate::vec::{Color, Point3, Ray};
use rand::{Rng, RngCore};
use rayon::prelude::*;

pub trait Background: Sync {
    fn color(&self, ray: &Ray) -> Color;
}

pub struct GradientBackground {
    top: Color,
    bottom: Color,
}

impl GradientBackground {
    pub fn new(top: Color, bottom: Color) -> GradientBackground {
        GradientBackground { top, bottom }
    }
    pub fn default() -> GradientBackground {
        let white: Color = Color::new(1.0f64, 1.0f64, 1.0f64);
        let blueish: Color = Color::new(0.5f64, 0.7f64, 1.0f64);

        GradientBackground::new(blueish, white)
    }
}

impl Background for GradientBackground {
    fn color(&self, ray: &Ray) -> Color {
        let unit_direction = ray.dir.unit();
        let t = 0.5f64 * (unit_direction.y() + 1.0f64);
        return (1.0 - t) * self.bottom + t * self.top;
    }
}

pub struct BlackBackground {}
impl BlackBackground {
    pub fn new() -> BlackBackground {
        BlackBackground {}
    }
}

impl Background for BlackBackground {
    fn color(&self, _: &Ray) -> Color {
        Color::ZERO
    }
}

#[derive(Copy, Clone)]
pub struct RenderingParams {
    pub samples_per_pixel: i32,
    pub image_height: usize,
    pub image_width: usize,
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

pub trait RayTracingAlgorithm: Sync {
    fn trace(&self, ray: &Ray, world: &dyn Hittable, background: &dyn Background, rng: &mut dyn RngCore) -> Color;
}

pub struct RecursiveRayTracer {
    pub max_depth: i32,
}

impl RecursiveRayTracer {
    fn trace_internal(
        &self,
        ray: &Ray,
        world: &dyn Hittable,
        background: &dyn Background,
        depth: i32,
        rng: &mut dyn RngCore,
    ) -> Color {
        if depth <= 0 {
            return Color::ZERO;
        }
        match world.hit(ray, 0.001, f64::INFINITY, rng) {
            Some(h) => match h.material.scatter(ray, &h, rng) {
                Some((attenuation, scattered)) => {
                    return attenuation * self.trace_internal(&scattered, world, background, depth - 1, rng);
                }
                None => {
                    return h.material.emit(h.u, h.v, h.p);
                }
            },
            None => background.color(ray),
        }
    }
}

impl RayTracingAlgorithm for RecursiveRayTracer {
    fn trace(&self, ray: &Ray, world: &dyn Hittable, background: &dyn Background, rng: &mut dyn RngCore) -> Color {
        self.trace_internal(ray, world, background, self.max_depth, rng)
    }
}

pub struct SingleLightSourceRayTracer {
    pub light_source: Point3,
    pub intensity: f64,
}

impl RayTracingAlgorithm for SingleLightSourceRayTracer {
    fn trace(&self, ray: &Ray, world: &dyn Hittable, background: &dyn Background, rng: &mut dyn RngCore) -> Color {
        match world.hit(ray, 0.001, f64::INFINITY, rng) {
            Some(hit) => match hit.material.scatter(ray, &hit, rng) {
                Some((attenuation, _)) => {
                    let l = (self.light_source - hit.p).unit();
                    let v = -ray.dir.unit();
                    let h = (l + v).unit();
                    // All materials are Lambertian.
                    let lambertian = attenuation * self.intensity * l.dot(hit.normal).max(0.0);
                    let blinn_phong = 0.5 * Color::ONE * self.intensity * h.dot(hit.normal).max(0.0).powi(100);
                    return lambertian + blinn_phong;
                }
                None => {
                    return hit.material.emit(hit.u, hit.v, hit.p);
                }
            },
            None => background.color(ray),
        }
    }
}

pub struct Renderer<'a, Tracer = RecursiveRayTracer, T = rngator::ThreadRngator>
where
    Tracer: RayTracingAlgorithm,
    T: rngator::Rngator,
{
    camera: &'a Camera,
    world: &'a dyn Hittable,
    background: &'a dyn Background,
    parameters: RenderingParams,
    tracer: Tracer,
    rng: T,
}

impl<'a, Tracer: RayTracingAlgorithm, T: rngator::Rngator> Renderer<'a, Tracer, T> {
    pub fn new_with_rng(
        camera: &'a Camera,
        world: &'a dyn Hittable,
        background: &'a dyn Background,
        parameters: RenderingParams,
        tracer: Tracer,
        rng: T,
    ) -> Renderer<'a, Tracer, T> {
        Renderer { camera, world, background, parameters, tracer, rng }
    }

    pub fn render_line(&self, j: usize, result: &mut [RGB], rng: &mut T::R) {
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
                let mut rng = self.rng.rng(j as u64);
                let mut line = vec![(0, 0, 0); self.parameters.image_width];
                self.render_line(j, line.as_mut_slice(), &mut rng);
                logger(j, self.parameters.image_height);
                line
            })
            .collect()
    }

    pub fn render_pixel(&self, i: usize, j: usize, rng: &mut T::R) -> RGB {
        let mut pixel_color = Color::ZERO;
        for _ in 0..self.parameters.samples_per_pixel {
            let u = ((i as f64) + rng.gen_range(0.0..1.0)) / (self.parameters.image_width as f64 - 1.0);
            let v = ((j as f64) + rng.gen_range(0.0..1.0)) / (self.parameters.image_height as f64 - 1.0);
            let r = self.camera.get_ray(u, v, rng);
            pixel_color = pixel_color + self.tracer.trace(&r, self.world, self.background, rng);
        }

        to_rgb(&pixel_color, self.parameters.samples_per_pixel)
    }
}
