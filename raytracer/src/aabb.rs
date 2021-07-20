pub use crate::ray::*;
pub use crate::vec3::*;

#[derive(Copy, Clone)]
pub struct AABB {
    pub _min: Vec3,
    pub _max: Vec3,
}

impl AABB {
    pub fn new(_min: &Vec3, _max: &Vec3) -> AABB {
        AABB { 
            _min: *_min,
            _max: *_max 
        }
    }

    pub fn hit(&self, _r: &Ray, tmin: f64, tmax: f64) -> bool {
        let mut t_min = tmin;
        let mut t_max = tmax;

        for i in 0..3 {
            let invd = 1.0 / _r.drc[i];
            let mut t0 = (self._min[i] - _r.org[i]) * invd;
            let mut t1 = (self._max[i] - _r.org[i]) * invd;
            if invd < 0.0 {
                std::mem::swap(&mut t1, &mut t0);
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }
}

pub fn surrounding_box(box0: AABB, box1: AABB) -> AABB {
    let small = Vec3::new(
        getmin(box0._min.x, box1._min.x),
        getmin(box0._min.y, box1._min.y),
        getmin(box0._min.z, box1._min.z)
    );
    let big = Vec3::new(
        getmax(box0._max.x, box1._max.x),
        getmax(box0._max.y, box1._max.y),
        getmax(box0._max.z, box1._max.z)
    );

    return AABB::new(&small, &big);
}

pub fn getmin(af: f64, bf: f64) -> f64 {
    return if af > bf {bf} else {af};
}

pub fn getmax(af: f64, bf: f64) -> f64 {
    return if af > bf {af} else {bf};
}
