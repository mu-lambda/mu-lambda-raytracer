use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::rngator;
use crate::vec::{Color, Ray};
use rand::Rng;
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
    fn color(&self, ray: &Ray) -> Color {
        Color::ZERO
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
    background: &'a dyn Background,
    parameters: RenderingParams,
    rng: T,
}

impl<'a> RayTracer<'a> {
    pub fn new(
        camera: &'a Camera,
        world: &'a dyn Hittable,
        background: &'a dyn Background,
        parameters: RenderingParams,
    ) -> RayTracer<'a, rngator::ThreadRngator> {
        RayTracer::new_with_rng(camera, world, background, parameters, rngator::ThreadRngator {})
    }
}

impl<'a, T: rngator::Rngator> RayTracer<'a, T> {
    pub fn new_with_rng(
        camera: &'a Camera,
        world: &'a dyn Hittable,
        background: &'a dyn Background,
        parameters: RenderingParams,
        rng: T,
    ) -> RayTracer<'a, T> {
        RayTracer { camera, world, background, parameters, rng }
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
                let mut rng = self.rng.rng();
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
            let u =
                ((i as f64) + rng.gen_range(0.0..1.0)) / (self.parameters.image_width as f64 - 1.0);
            let v = ((j as f64) + rng.gen_range(0.0..1.0))
                / (self.parameters.image_height as f64 - 1.0);
            let r = self.camera.get_ray(u, v, rng);
            pixel_color =
                pixel_color + self.ray_color(&r, self.world, self.parameters.max_depth, rng);
        }

        to_rgb(&pixel_color, self.parameters.samples_per_pixel)
    }

    fn ray_color(&self, ray: &Ray, world: &dyn Hittable, depth: i32, rng: &mut T::R) -> Color {
        if depth <= 0 {
            return Color::ZERO;
        }
        match world.hit(ray, 0.001, f64::INFINITY) {
            Some(h) => match h.material.scatter(ray, &h, rng) {
                Some((attenuation, scattered)) => {
                    return attenuation * self.ray_color(&scattered, world, depth - 1, rng);
                }
                None => {
                    return h.material.emit(h.u, h.v, h.p);
                }
            },
            None => self.background.color(ray),
        }
    }
}
