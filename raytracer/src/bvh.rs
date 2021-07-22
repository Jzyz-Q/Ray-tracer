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
}

impl BvhNode {
    pub fn new_list(list: Hlist, time0: f64, time1: f64) -> Self {
        let len = list.objects.len();
        BvhNode::new(list.objects, 0, len, time0, time1)
    }

    pub fn new(
        mut objects: Vec<Arc<dyn Object>>,
        start: usize,
        end: usize,
        time0: f64,
        time1: f64,
    ) -> Self {
        let axis = rand::thread_rng().gen_range(0..3);
        let left;
        let right;
        let _box;

        let comparator = match axis {
            0 => Self::box_x_compare,
            1 => Self::box_y_compare,
            _ => Self::box_z_compare,
        };

        let len = end - start;

        match len {
            1 => {
                left = objects[start].clone();
                right = objects[start].clone();
            }
            2 => {
                if comparator(&objects[start], &objects[start + 1]) == Ordering::Less {
                    right = objects[start + 1].clone();
                    left = objects[start].clone();
                } else {
                    left = objects[start + 1].clone();
                    right = objects[start].clone();
                }
            }
            _ => {
                let obj = &mut objects[start..end];
                obj.sort_by(|a, b| comparator(a, b));

                let mid = (start + end) / 2;
                left = Arc::new(BvhNode::new(objects.clone(), start, mid, time0, time1));
                right = Arc::new(BvhNode::new(objects.clone(), mid, end, time0, time1));
            }
        }

        if let Some(box_left) = left.bounding_box(time0, time1) {
            if let Some(box_right) = right.bounding_box(time0, time1) {
                _box = surrounding_box(box_left, box_right);
                return Self { left, right, _box };
            }
        }
        panic!();
    }

    fn box_x_compare(a: &Arc<dyn Object>, b: &Arc<dyn Object>) -> Ordering {
        if let Some(box_a) = a.bounding_box(0.0, 0.0) {
            if let Some(box_b) = b.bounding_box(0.0, 0.0) {
                if let Some(tmp) = box_a._min.x.partial_cmp(&box_b._min.x) {
                    return tmp;
                }
            }
        }
        panic!();
    }
    fn box_y_compare(a: &Arc<dyn Object>, b: &Arc<dyn Object>) -> Ordering {
        if let Some(box_a) = a.bounding_box(0.0, 0.0) {
            if let Some(box_b) = b.bounding_box(0.0, 0.0) {
                if let Some(tmp) = box_a._min.y.partial_cmp(&box_b._min.y) {
                    return tmp;
                }
            }
        }
        panic!();
    }
    fn box_z_compare(a: &Arc<dyn Object>, b: &Arc<dyn Object>) -> Ordering {
        if let Some(box_a) = a.bounding_box(0.0, 0.0) {
            if let Some(box_b) = b.bounding_box(0.0, 0.0) {
                if let Some(tmp) = box_a._min.z.partial_cmp(&box_b._min.z) {
                    return tmp;
                }
            }
        }
        panic!();
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
}
