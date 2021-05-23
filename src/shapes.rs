use crate::bhv::AABB;
use crate::datatypes::{dot, Point3, Ray, Vec3};
use crate::hittable::{Hit, Hittable};
use crate::materials::Material;

pub struct Sphere<T: Material> {
    center: Point3,
    radius: f64,
    material: T,
}

impl<T: Material> Sphere<T> {
    pub fn new(center: Point3, radius: f64, material: T) -> Sphere<T> {
        Sphere { center, radius, material }
    }
    pub fn center(&self) -> Point3 {
        self.center
    }
    pub fn radius(&self) -> f64 {
        self.radius
    }
}

impl<T: Material> Hittable for Sphere<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let oc = &r.orig - &self.center;
        let a = r.dir.length_squared();
        let half_b = dot(oc, r.dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let t = root;
        let p = r.at(t);
        let normal = (p - self.center) / self.radius;
        return Some(Hit::new_with_face_normal(&p, t, &normal, r, &self.material));
    }

    fn bounding_box(&self) -> Option<AABB> {
        let rad_v = Vec3::new(self.radius, self.radius, self.radius);
        Some(AABB::new(self.center - rad_v, self.center + rad_v))
    }
}
