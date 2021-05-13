use crate::datatypes::{dot, Point3, Ray, Vec3};
use std::option::Option;
use std::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    fn new_with_face_normal(p: &Point3, t: f64, outward_normal: &Vec3, r: &Ray) -> HitRecord {
        let front_face = dot(r.dir, *outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal.clone()
        } else {
            -outward_normal
        };
        return HitRecord {
            p: *p,
            normal,
            t,
            front_face,
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Sphere {
    center: Point3,
    radius: f64,
}

impl Sphere {
    pub fn new(center: &Point3, radius: f64) -> Sphere {
        Sphere {
            center: *center,
            radius,
        }
    }
    pub fn center(&self) -> Point3 {
        self.center
    }
    pub fn radius(&self) -> f64 {
        self.radius
    }
}

impl Hittable for Sphere {
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
        return Some(HitRecord::new_with_face_normal(&p, t, &normal, r));
    }
}

pub struct HittableList {
    contents: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList {
            contents: Vec::new(),
        }
    }
    pub fn push(&mut self, v: Box<dyn Hittable>) {
        self.contents.push(v);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut result: Option<HitRecord> = None;
        let mut closest_so_far = t_max;

        for o in self.contents.iter() {
            match o.hit(r, t_min, closest_so_far) {
                Some(h) => {
                    result = Some(h);
                    closest_so_far = h.t
                }
                None => {}
            }
        }
        return result;
    }
}
