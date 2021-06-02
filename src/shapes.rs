use crate::aarects::{AARect, Axis};
use crate::bhv::{Bounded, AABB};
use crate::hittable::{Hit, Hittable};
use crate::materials::Material;
use crate::vec::{Point3, Ray, Vec3};

pub struct Empty {}

impl Empty {
    pub const INSTANCE: Empty = Empty {};
}
impl Hittable for Empty {
    fn hit(&self, _: &Ray, _: f64, _: f64) -> Option<Hit> {
        None
    }
}

impl Bounded for Empty {
    fn bounding_box(&self) -> AABB {
        AABB::new(Point3::ZERO, Point3::ZERO)
    }
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

fn sphere_uv(normal: &Vec3) -> (f64, f64) {
    // normal: a given point on the sphere of radius one, centered at the origin.
    // u: returned value [0,1] of angle around the Y axis from X=-1.
    // v: returned value [0,1] of angle from Y=-1 to Y=+1.
    //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
    //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
    //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>
    let theta = (-normal.y()).acos();
    let phi = (-normal.z()).atan2(normal.x()) + std::f64::consts::PI;

    (phi / (2.0 * std::f64::consts::PI), theta / std::f64::consts::PI)
}

impl<T: Material + Sync> Hittable for Sphere<T> {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<Hit> {
        let oc = &r.orig - &self.center;
        let a = r.dir.length_squared();
        let half_b = oc.dot(r.dir);
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
        let (u, v) = sphere_uv(&normal);
        Some(Hit::new_with_face_normal(&p, t, u, v, &normal, r, &self.material))
    }
}

impl<T: Material + Sync> Bounded for Sphere<T> {
    fn bounding_box(&self) -> AABB {
        let rad_v = Vec3::new(self.radius, self.radius, self.radius);
        AABB::new(self.center - rad_v, self.center + rad_v)
    }
}

pub struct XYRect<T: Material> {
    r: AARect,
    material: T,
}

impl<T: Material> XYRect<T> {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, z: f64, material: T) -> XYRect<T> {
        let r = AARect::new(Axis::X, x0, x1, Axis::Y, y0, y1, z);
        XYRect { r, material }
    }
}

impl<T: Material + Sync> Hittable for XYRect<T> {
    fn hit(&self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        self.r.hit(r, tmin, tmax, &self.material)
    }
}

impl<T: Material + Sync> Bounded for XYRect<T> {
    fn bounding_box(&self) -> AABB {
        self.r.bounding_box()
    }
}

pub struct XZRect<T: Material> {
    r: AARect,
    material: T,
}

impl<T: Material> XZRect<T> {
    pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, y: f64, material: T) -> XYRect<T> {
        let r = AARect::new(Axis::X, x0, x1, Axis::Z, z0, z1, y);
        XYRect { r, material }
    }
}

impl<T: Material + Sync> Hittable for XZRect<T> {
    fn hit(&self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        self.r.hit(r, tmin, tmax, &self.material)
    }
}

impl<T: Material + Sync> Bounded for XZRect<T> {
    fn bounding_box(&self) -> AABB {
        self.r.bounding_box()
    }
}

pub struct YZRect<T: Material> {
    r: AARect,
    material: T,
}

impl<T: Material> YZRect<T> {
    pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, x: f64, material: T) -> XYRect<T> {
        let r = AARect::new(Axis::Y, y0, y1, Axis::Z, z0, z1, x);
        XYRect { r, material }
    }
}

impl<T: Material + Sync> Hittable for YZRect<T> {
    fn hit(&self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        self.r.hit(r, tmin, tmax, &self.material)
    }
}

impl<T: Material + Sync> Bounded for YZRect<T> {
    fn bounding_box(&self) -> AABB {
        self.r.bounding_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_uv() {
        assert_eq!((0.5, 0.5), sphere_uv(&Vec3::new(1.0, 0.0, 0.0)));
        assert_eq!((0.5, 1.0), sphere_uv(&Vec3::new(0.0, 1.0, 0.0)));
        assert_eq!((0.25, 0.5), sphere_uv(&Vec3::new(0.0, 0.0, 1.0)));

        assert_eq!((0.0, 0.5), sphere_uv(&Vec3::new(-1.0, 0.0, 0.0)));
        assert_eq!((0.5, 0.0), sphere_uv(&Vec3::new(0.0, -1.0, 0.0)));
        assert_eq!((0.75, 0.5), sphere_uv(&Vec3::new(0.0, 0.0, -1.0)));
    }
}
