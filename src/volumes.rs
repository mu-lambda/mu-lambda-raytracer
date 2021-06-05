use crate::hittable::{Hit, Hittable};
use crate::materials::Material;
use crate::textures::{SolidColor, Texture};
use crate::vec::{Color, Ray, Vec3};
use rand::Rng;

pub struct ConstantMedium<O: Hittable, T: Texture> {
    boundary: O,
    phase_function: Isotropic<T>,
    neg_inv_density: f64,
}

impl<O: Hittable, T: Texture> ConstantMedium<O, T> {
    pub fn new(boundary: O, d: f64, texture: T) -> ConstantMedium<O, T> {
        ConstantMedium { boundary, neg_inv_density: -1.0 / d, phase_function: Isotropic::new(texture) }
    }
}

impl<O: Hittable> ConstantMedium<O, SolidColor> {
    pub fn from_color(boundary: O, d: f64, color: Color) -> ConstantMedium<O, SolidColor> {
        ConstantMedium::new(boundary, d, SolidColor::from_color(color))
    }
}

impl<O: Hittable, T: Texture> Hittable for ConstantMedium<O, T> {
    fn hit<'a>(&'a self, r: &Ray, t_min: f64, t_max: f64, rng: &mut dyn rand::RngCore) -> Option<Hit<'a>> {
        let mut h1 = match self.boundary.hit(r, f64::NEG_INFINITY, f64::INFINITY, rng) {
            None => return None,
            Some(h) => h,
        };
        let mut h2 = match self.boundary.hit(r, h1.t + 0.001, f64::INFINITY, rng) {
            None => return None,
            Some(h) => h,
        };

        h1.t = h1.t.max(t_min);
        h2.t = h2.t.min(t_max);

        if h1.t >= h2.t {
            return None;
        }

        h1.t = h1.t.max(0.0);

        let ray_scale = r.dir.length();
        let distance_inside_bondary = (h2.t - h1.t) * ray_scale;
        let hit_distance = self.neg_inv_density * rng.gen_range(0.0f64..1.0f64).ln();

        if hit_distance > distance_inside_bondary {
            return None;
        }

        let t = h1.t + hit_distance / ray_scale;
        let p = r.at(t);
        Some(Hit {
            p,
            t,
            u: 0.0,
            v: 0.0,
            normal: Vec3::new(1.0, 0.0, 0.0),
            front_face: true,
            material: &self.phase_function,
        })
    }
}

pub struct Isotropic<T: Texture> {
    albedo: T,
}

impl<T: Texture> Isotropic<T> {
    pub fn new(albedo: T) -> Isotropic<T> {
        Isotropic { albedo }
    }
}

impl<T: Texture> Material for Isotropic<T> {
    fn scatter(&self, _: &Ray, h: &Hit, rng: &mut dyn rand::RngCore) -> Option<(Color, Ray)> {
        let scattered = Ray::new(h.p, Vec3::random_in_unit_sphere(rng));
        let attenuation = self.albedo.value(h.u, h.v, h.p);
        Some((attenuation, scattered))
    }
}
