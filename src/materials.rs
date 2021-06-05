use crate::hittable;
use crate::textures::Texture;
use crate::vec::{Color, Point3, Ray, Vec3};
use rand::Rng;

pub trait Material: Sync {
    fn scatter(&self, ray: &Ray, h: &hittable::Hit, rng: &mut dyn rand::RngCore) -> Option<(Color, Ray)>;

    fn emit(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        Color::ZERO
    }
}

#[derive(Copy, Clone)]
pub struct Lambertian<T: Texture> {
    pub albedo: T,
}

impl<T: Texture> Lambertian<T> {
    pub fn new(albedo: T) -> Lambertian<T> {
        Lambertian { albedo }
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, _ray: &Ray, h: &hittable::Hit, rng: &mut dyn rand::RngCore) -> Option<(Color, Ray)> {
        let mut scatter_direction = h.normal + Vec3::random_in_hemisphere(&h.normal, rng);
        if scatter_direction.near_zero() {
            scatter_direction = h.normal;
        }
        let attenuation = self.albedo.value(h.u, h.v, h.p);
        return Some((attenuation, Ray::new(h.p, scatter_direction)));
    }
}

#[derive(Copy, Clone)]
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
    v - 2.0 * v.dot(n) * n
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, h: &hittable::Hit, rng: &mut dyn rand::RngCore) -> Option<(Color, Ray)> {
        let reflected = reflect(ray.dir.unit(), h.normal);
        let scattered = Ray::new(h.p, reflected + self.fuzz * Vec3::random_in_unit_sphere(rng));
        if scattered.dir.dot(h.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

#[derive(Copy, Clone)]
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
    fn scatter(&self, ray: &Ray, h: &hittable::Hit, rng: &mut dyn rand::RngCore) -> Option<(Color, Ray)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if !h.front_face { self.index_of_refraction } else { 1.0 / self.index_of_refraction };

        let unit_direction = ray.dir.unit();
        let cos_theta = h.normal.dot(-unit_direction).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.gen_range(0.0..1.0) {
            reflect(unit_direction, h.normal)
        } else {
            refract(unit_direction, h.normal, refraction_ratio)
        };

        return Some((attenuation, Ray::new(h.p, direction)));
    }
}

#[derive(Clone)]
pub struct DiffuseLight<T: Texture> {
    texture: T,
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new(texture: T) -> DiffuseLight<T> {
        DiffuseLight { texture }
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn scatter(&self, _: &Ray, _: &hittable::Hit, _: &mut dyn rand::RngCore) -> Option<(Color, Ray)> {
        None
    }

    fn emit(&self, u: f64, v: f64, p: Point3) -> Color {
        self.texture.value(u, v, p)
    }
}
