use crate::datatypes::{dot, Point3, Ray, Vec3};
use crate::materials::Material;
use std::option::Option;
use std::vec::Vec;

#[derive(Clone)]
pub struct HitRecord<'a> {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    fn new_with_face_normal(
        p: &Point3,
        t: f64,
        outward_normal: &Vec3,
        r: &Ray,
        material: &'a dyn Material,
    ) -> HitRecord<'a> {
        let front_face = dot(r.dir, *outward_normal) < 0.0;
        let normal = if front_face { *outward_normal } else { -outward_normal };
        return HitRecord { p: *p, normal, t, front_face, material };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
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
        return Some(HitRecord::new_with_face_normal(&p, t, &normal, r, &self.material));
    }
}

pub struct HittableList<'a> {
    contents: Vec<Box<dyn Hittable + 'a>>,
}

impl<'a> HittableList<'a> {
    pub fn new() -> HittableList<'a> {
        HittableList { contents: Vec::new() }
    }
    pub fn push<T: Hittable + 'a>(&mut self, v: T) {
        self.contents.push(Box::new(v));
    }
}

impl Hittable for HittableList<'_> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut result: Option<HitRecord> = None;
        let mut closest_so_far = t_max;

        for o in self.contents.iter() {
            match o.hit(r, t_min, closest_so_far) {
                Some(h) => {
                    closest_so_far = h.t;
                    result = Some(h);
                }
                None => {}
            }
        }
        return result;
    }
}
