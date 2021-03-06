use crate::materials::Material;
use crate::vec::{Point3, Ray, Vec3};
use std::option::Option;
use std::vec::Vec;

#[derive(Clone)]
pub struct Hit<'a> {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
    pub material: &'a dyn Material,
}

impl<'a> Hit<'a> {
    pub fn new_with_face_normal(
        p: &Point3,
        t: f64,
        u: f64,
        v: f64,
        outward_normal: &Vec3,
        r: &Ray,
        material: &'a dyn Material,
    ) -> Hit<'a> {
        let front_face = outward_normal.dot(r.dir) < 0.0;
        let normal = if front_face { *outward_normal } else { -outward_normal };
        return Hit { p: *p, normal, t, u, v, front_face, material };
    }
}

pub trait Hittable: Sync {
    fn hit<'a>(&'a self, r: &Ray, t_min: f64, t_max: f64, rng: &mut dyn rand::RngCore) -> Option<Hit<'a>>;
}

pub struct HittableList<'a> {
    contents: Vec<Box<dyn Hittable + 'a>>,
}

impl<'a> HittableList<'a> {
    pub fn new() -> HittableList<'a> {
        HittableList { contents: Vec::new() }
    }
    pub fn add<T: Hittable + 'a>(&mut self, v: T) {
        self.contents.push(Box::new(v));
    }
    pub fn push<T: Hittable + 'a>(&mut self, v: Box<T>) {
        self.contents.push(v);
    }
}

impl<'a> Hittable for HittableList<'a> {
    fn hit<'b>(&'b self, r: &Ray, t_min: f64, t_max: f64, rng: &mut dyn rand::RngCore) -> Option<Hit<'b>> {
        let mut result: Option<Hit> = None;
        let mut closest_so_far = t_max;

        for o in self.contents.iter() {
            match o.hit(r, t_min, closest_so_far, rng) {
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
