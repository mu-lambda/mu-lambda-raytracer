use crate::datatypes::{dot, unit_vector, Color, Point3, Ray, Vec3};
use crate::hittable;
use std::option;

pub trait Material {
    fn scatter(&self, ray: &Ray, h: &hittable::HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, h: &hittable::HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = h.normal + Vec3::random_in_hemisphere(&h.normal);
        if scatter_direction.near_zero() {
            scatter_direction = h.normal;
        }
        return Some((self.albedo, Ray::new(h.p, scatter_direction)));
    }
}

pub struct Metal {
    pub albedo: Color,
}
impl Metal {
    pub fn new(albedo: Color) -> Metal {
        Metal { albedo }
    }
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * dot(v, n) * n
}
impl Material for Metal {
    fn scatter(&self, ray: &Ray, h: &hittable::HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(unit_vector(&ray.dir), h.normal);
        let scattered = Ray::new(h.p, reflected);
        if dot(scattered.dir, h.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}
