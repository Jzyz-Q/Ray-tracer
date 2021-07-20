/* 
if (ray hits bounding object)
    return whether ray hits bounded objects
else
    return false 
*/
pub use crate::aabb::*;
pub use crate::hittable::*;
pub use rand::Rng;
pub use std::{cmp::Ordering, sync::Arc};

#[derive(Clone)]
pub struct BvhNode {
    left: Arc<dyn Object>,
    right: Arc<dyn Object>,
    _box: AABB,
    if_dark: bool,
}

impl BvhNode {
    pub fn new(list: Hlist, time0: f64, time1: f64) -> Arc<dyn Object> {
        BvhNode::initial(list.objects, time0, time1, list.dark_flag)
    }

    pub fn initial(mut objects: Vec<Arc<dyn Object>>, time0: f64, time1: f64, if_dark: bool) -> Arc<dyn Object> {
        let axis = rand::thread_rng().gen_range(0..3);

        match objects.len() {
            0 => {
                panic!("no object");
            }
            1 => objects.remove(0),
            _ => {
                objects.sort_by(|a, b| {
                    a.bounding_box(time0, time1).unwrap()._min[axis].partial_cmp(
                    &b.bounding_box(time0, time1).unwrap()._min[axis]).unwrap()
                });

                let mut left_objects = objects;
                let right_objects = left_objects.split_off(left_objects.len() / 2);
                let left = Self::initial(left_objects, time0, time1, if_dark);
                let right = Self::initial(right_objects, time0, time1, if_dark);
                let _box = surrounding_box(left.bounding_box(time0, time1).unwrap(),
                                           right.bounding_box(time0, time1).unwrap());
                Arc::new(Self {
                    left,
                    right,
                    _box,
                    if_dark
                })
            }
        }
    }
}

impl Object for BvhNode {
    fn hit(&self, _r: &Ray, t_min: f64, t_max: f64) -> Option<Hitrecord> {
        match self._box.hit(_r, t_min, t_max) {
            true => {
                let l = self.left.hit(_r, t_min, t_max);
                let r = self.right.hit(_r, t_min, t_max);
                match (l, r) {
                    (Some(l), Some(r)) => {
                        if l.t < r.t {
                            Some(l)
                        } else {
                            Some(r)
                        }
                    }
                    (Some(l), None) => Some(l),
                    (None, Some(r)) => Some(r),
                    (None, None) => None,
                }
            }
            false => None,
        }
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(self._box)
    }

    // fn get_background(&self, t: f64) -> Color {
    //     if self.if_dark {
    //         Color::zero()
    //     } else {
    //         Color::ones() * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
    //     }
    // }
}

