use crate::bhv::{Bounded, AABB};
use crate::hittable::{Hit, Hittable};
use crate::vec::{Ray, Vec3};

pub struct Translate<T>
where
    T: Hittable,
{
    original: T,
    offset: Vec3,
}

impl<T> Translate<T>
where
    T: Hittable,
{
    pub fn new(original: T, offset: Vec3) -> Translate<T> {
        Translate { original, offset }
    }
}

impl<T> Hittable for Translate<T>
where
    T: Hittable,
{
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

impl<T> Bounded for Translate<T>
where
    T: Bounded,
{
    fn bounding_box(&self) -> AABB {
        let aabb = self.original.bounding_box();
        AABB::new(aabb.min() + self.offset, aabb.max() + self.offset)
    }
}
