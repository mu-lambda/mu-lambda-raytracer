use crate::datatypes::{dot, unit_vector, Color, Ray, Vec3};
use crate::hittable;
use rand::Rng;

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
    pub fuzz: f64,
}
impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal { albedo, fuzz }
    }
}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * dot(v, n) * n
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, h: &hittable::HitRecord) -> Option<(Color, Ray)> {
        let reflected = reflect(unit_vector(&ray.dir), h.normal);
        let scattered = Ray::new(h.p, reflected + self.fuzz * Vec3::random_in_unit_sphere());
        if dot(scattered.dir, h.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = dot(-uv, n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

pub struct Dielectric {
    pub index_of_refraction: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Dielectric {
        Dielectric { index_of_refraction }
    }
}

fn reflectance(cos_theta: f64, refraction_ratio: f64) -> f64 {
    // Use Schlick's approximation for reflectance.
    let r0 = (1.0 - refraction_ratio) / (1.0 + refraction_ratio);
    let r0_sq = r0 * r0;
    r0_sq + (1.0 - r0_sq) * (1.0 - cos_theta).powi(5)
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, h: &hittable::HitRecord) -> Option<(Color, Ray)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio =
            if !h.front_face { self.index_of_refraction } else { 1.0 / self.index_of_refraction };

        let unit_direction = unit_vector(&ray.dir);
        let cos_theta = dot(-unit_direction, h.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract
            || reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen_range(0.0..1.0)
        {
            reflect(unit_direction, h.normal)
        } else {
            refract(unit_direction, h.normal, refraction_ratio)
        };

        return Some((attenuation, Ray::new(h.p, direction)));
    }
}
