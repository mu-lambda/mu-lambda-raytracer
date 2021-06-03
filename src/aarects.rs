use crate::bhv::AABB;
use crate::hittable::Hit;
use crate::materials::Material;
use crate::vec::{Point3, Ray, Vec3};

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

fn index(a: Axis) -> usize {
    match a {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}

fn other(a0: Axis, a1: Axis) -> Axis {
    match (a0, a1) {
        (Axis::X, Axis::Y) => Axis::Z,
        (Axis::X, Axis::Z) => Axis::Y,
        (Axis::Y, Axis::Z) => Axis::X,
        _ => panic!(),
    }
}

// Axis-aligned rect
#[derive(Clone, Copy)]
pub struct AARect {
    a0: usize,
    a1: usize,
    aplane: usize,

    a0_v0: f64,
    a0_v1: f64,
    a1_v0: f64,
    a1_v1: f64,
    aplane_v: f64,
}

impl AARect {
    pub fn new(
        a0: Axis,
        a0_v0: f64,
        a0_v1: f64,
        a1: Axis,
        a1_v0: f64,
        a1_v1: f64,
        aplane_v: f64,
    ) -> AARect {
        AARect {
            a0: index(a0),
            a1: index(a1),
            aplane: index(other(a0, a1)),

            a0_v0: a0_v0.min(a0_v1),
            a0_v1: a0_v1.max(a0_v0),
            a1_v0: a1_v0.min(a1_v0),
            a1_v1: a1_v1.max(a1_v0),
            aplane_v,
        }
    }

    pub fn hit<'a>(
        &self,
        r: &Ray,
        tmin: f64,
        tmax: f64,
        material: &'a dyn Material,
    ) -> Option<Hit<'a>> {
        let t = (self.aplane_v - r.orig.e[self.aplane]) / r.dir.e[self.aplane];
        if t < tmin || t > tmax {
            return None;
        }

        let a0_v = r.orig.e[self.a0] + t * r.dir.e[self.a0];
        let a1_v = r.orig.e[self.a1] + t * r.dir.e[self.a1];

        if a0_v < self.a0_v0 || a0_v > self.a0_v1 || a1_v < self.a1_v0 || a1_v > self.a1_v1 {
            return None;
        }

        let u = (a0_v - self.a0_v0) / (self.a0_v1 - self.a0_v0);
        let v = (a1_v - self.a1_v0) / (self.a1_v1 - self.a1_v0);
        let mut outward_normal = Vec3::ZERO;
        outward_normal.e[self.aplane] = 1.0;

        Some(Hit::new_with_face_normal(&r.at(t), t, u, v, &outward_normal, r, material))
    }

    pub fn bounding_box(&self) -> AABB {
        let mut minimum = Point3::ZERO;
        let mut maximum = Point3::ZERO;
        minimum.e[self.a0] = self.a0_v0;
        minimum.e[self.a1] = self.a1_v0;
        minimum.e[self.aplane] = self.aplane_v - 0.001;
        maximum.e[self.a0] = self.a0_v0;
        maximum.e[self.a1] = self.a1_v0;
        maximum.e[self.aplane] = self.aplane_v + 0.001;

        AABB::new(minimum, maximum)
    }
}
