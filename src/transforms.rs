use crate::bhv::{Bounded, AABB};
use crate::hittable::{Hit, Hittable};
use crate::vec::{Point3, Ray, Vec3};

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub fn index(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}

pub struct Translate<T: Hittable> {
    original: T,
    offset: Vec3,
}

impl<T: Hittable> Translate<T> {
    pub fn new(original: T, offset: Vec3) -> Translate<T> {
        Translate { original, offset }
    }
}

impl<T: Hittable> Hittable for Translate<T> {
    fn hit<'a>(&'a self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit<'a>> {
        let moved_r = Ray { orig: r.orig - self.offset, dir: r.dir };

        match self.original.hit(&moved_r, t_min, t_max) {
            None => None,
            Some(h) => Some(Hit::new_with_face_normal(
                &(h.p + self.offset),
                h.t,
                h.u,
                h.v,
                &h.normal,
                &moved_r,
                h.material,
            )),
        }
    }
}

impl<T: Bounded> Bounded for Translate<T> {
    fn bounding_box(&self) -> AABB {
        let aabb = self.original.bounding_box();
        AABB::new(aabb.min() + self.offset, aabb.max() + self.offset)
    }
}

pub struct Rotate<T: Bounded> {
    a1: usize,
    sin_theta: f64,
    cos_theta: f64,
    bounding_box: AABB,
    original: T,
}

impl<T: Bounded> Rotate<T> {
    pub fn new(original: T, axis: Axis, angle: f64) -> Rotate<T> {
        let a1 = index(axis); // if this is Y...
        let a2 = (a1 + 1) % 3; // ...this is Z...
        let a0 = (a1 + 2) % 3; // ...and this is X

        let theta = angle * std::f64::consts::PI / 180.0;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let b = original.bounding_box();
        let mut min = Point3 { e: [std::f64::NEG_INFINITY; 3] };
        let mut max = Point3 { e: [std::f64::NEG_INFINITY; 3] };

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let a0_v = if i == 1 { b.max().e[a0] } else { b.min().e[a0] };
                    let a1_v = if j == 1 { b.max().e[a1] } else { b.min().e[a1] };
                    let a2_v = if k == 1 { b.max().e[a2] } else { b.min().e[a2] };

                    let new_a0_v = cos_theta * a0_v + sin_theta * a2_v;
                    let new_a2_v = -sin_theta * a0_v + cos_theta * a2_v;

                    let mut tester = [0.0f64; 3];
                    tester[a0] = new_a0_v;
                    tester[a1] = a1_v;
                    tester[a2] = new_a2_v;

                    for c in 0..3 {
                        min.e[c] = min.e[c].min(tester[c]);
                        max.e[c] = max.e[c].max(tester[c]);
                    }
                }
            }
        }

        Rotate { a1, sin_theta, cos_theta, original, bounding_box: AABB::new(min, max) }
    }

    fn a0(&self) -> usize {
        (self.a1 + 2) % 3
    }
    fn a2(&self) -> usize {
        (self.a1 + 1) % 3
    }

    fn rotate_back(&self, v: &Vec3) -> Vec3 {
        let a0 = self.a0();
        let a2 = self.a2();

        let mut result = *v;
        result.e[a0] = self.cos_theta * v.e[a0] - self.sin_theta * v.e[a2];
        result.e[a2] = self.sin_theta * v.e[a0] + self.cos_theta * v.e[a2];
        result
    }

    fn rotate(&self, v: &Vec3) -> Vec3 {
        let a0 = self.a0();
        let a2 = self.a2();

        let mut result = *v;
        result.e[a0] = self.cos_theta * v.e[a0] + self.sin_theta * v.e[a2];
        result.e[a2] = -self.sin_theta * v.e[a0] + self.cos_theta * v.e[a2];
        result
    }
}

impl<T: Bounded> Hittable for Rotate<T> {
    fn hit<'a>(&'a self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit<'a>> {
        let o = self.rotate_back(&r.orig);
        let d = self.rotate_back(&r.dir);

        let rotated_r = Ray::new(o, d);
        match self.original.hit(&rotated_r, t_min, t_max) {
            None => None,
            Some(h) => {
                let p = self.rotate(&h.p);
                let normal = self.rotate(&h.normal);
                Some(Hit::new_with_face_normal(&p, h.t, h.u, h.v, &normal, &rotated_r, h.material))
            }
        }
    }
}

impl<T: Bounded> Bounded for Rotate<T> {
    fn bounding_box(&self) -> AABB {
        self.bounding_box
    }
}
