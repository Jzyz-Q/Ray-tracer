#![allow(warnings, unused)]

use crate::{material::Material, ray::Ray, vec3::Vec3, aabb::AABB};
use std::ops::Mul;
pub use std::sync::Arc;
use crate::aabb::surrounding_box;

pub trait Object: Send + Sync {
    fn hit(&self, ray: &Ray, t1_min: f64, t1_max: f64) -> Option<Hitrecord>;
    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB>;
    //fn get_background(&self, t: f64) -> Color;
}

#[derive(Clone)]
pub struct Hitrecord {
    pub p: Vec3,
    pub n: Vec3,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool, //法相与入射方向相反
    pub mat_ptr: Arc<dyn Material>,
}

impl Hitrecord {
    pub fn new(p: Vec3, n: Vec3, t_in: f64, m: Arc<dyn Material>) -> Self {
        Self {
            p: p,
            n: n,
            mat_ptr: m,
            t: t_in,
            u: 0.0,
            v: 0.0,
            front_face: true,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        //判断入射面方向

        // ray is outside the sphere
        self.front_face = (r.drc * outward_normal) < 0.0;

        // ray outside <=> n = outward_normal
        self.n = if self.front_face {
            outward_normal
        }
        ////////////?????????
        else {
            -outward_normal
        };
    }

    pub fn set_uv(&mut self, res: (f64, f64)) {
        self.u = res.0;
        self.v = res.1;
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub ct: Vec3,
    pub rd: f64,
    pub mat_ptr: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, m: Arc<dyn Material>) -> Sphere {
        Sphere {
            ct: center,
            rd: radius,
            mat_ptr: m,
        }
    }
}

impl Object for Sphere {
    // 射入面判断
    fn hit(&self, r: &Ray, t1_min: f64, t1_max: f64) -> Option<Hitrecord> {
        let oc = r.org - self.ct;
        let a = r.drc.squared_length();
        let b = oc.mul(r.drc);
        let c = oc.squared_length() - self.rd * self.rd;
        let dis = b * b - a * c;

        if (dis > 0.0) {
            let root = dis.sqrt();

            let mut temp = (0.0 - b - root) / a;
            if temp < t1_max && temp > t1_min {
                let temp_p = r.at(temp);
                let temp_n: Vec3 = (temp_p - self.ct).unit();
                let outward_normal = (temp_p - self.ct) / self.rd;
                let mut rec = Hitrecord::new(temp_p, temp_n, temp, self.mat_ptr.clone());

                rec.set_face_normal(&r, outward_normal);
                let res = get_sphere_uv(&((rec.p - self.ct)/self.rd));
                rec.set_uv(res);
                return Some(rec);
            }

            temp = (0.0 - b + root) / a;
            if temp < t1_max && temp > t1_min {
                let temp_p = r.at(temp);
                let temp_n: Vec3 = (temp_p - self.ct).unit();
                let outward_normal = (temp_p - self.ct) / self.rd;
                let mut rec = Hitrecord::new(temp_p, temp_n, temp, self.mat_ptr.clone());

                rec.set_face_normal(&r, outward_normal);
                let res = get_sphere_uv(&rec.p);
                rec.set_uv(res);
                return Some(rec);
            }
        }

        return None;
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        let output_box = AABB::new(
            &(self.ct - Vec3::ones() * self.rd),
            &(self.ct + Vec3::ones() * self.rd),
        );
        Some(output_box)
    }
}

pub fn get_sphere_uv(p: &Vec3) -> (f64, f64) {
    let pi = std::f64::consts::PI;
    let phi = p.z.atan2(p.x);
    let theta = p.y.asin();
    let u = 1.0 - (phi + pi) / (2.0 * pi);
    let v = (theta + pi / 2.0) / pi;
    (u, v)
}

#[derive(Clone)]
pub struct Hlist {
    pub objects: Vec<Arc<dyn Object>>,
    pub dark_flag: bool,
}

impl Hlist {
    pub fn new(dark_flag: bool) -> Hlist {
        Hlist {
            objects: std::vec::Vec::new(),
            dark_flag,
        }
    }

    pub fn push(&mut self, ob: Arc<dyn Object>) {
        self.objects.push(ob);
    }
}

impl Object for Hlist {
    fn hit(&self, r: &Ray, t1_min: f64, t1_max: f64) -> Option<Hitrecord> {
        let mut limit = t1_max;
        let mut score: Option<Hitrecord> = None;

        for object in self.objects.iter() {
            let rec = object.hit(r, t1_min, limit);
            if let Some(h) = rec {
                limit = h.t;
                score = Some(h).clone();
            }
        }
        return score;
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }

        let mut output_box = AABB::new(&Vec3::zero(), &Vec3::zero());
        let mut first_box: bool = true;
        
        for object in self.objects.iter() {
            let rec = object.bounding_box(t0, t1);
            match rec {
                Some(val) => {
                    output_box = if first_box {
                        val
                    } else {
                        surrounding_box(output_box, val)
                    };
                    first_box = false;
                }
                None => {
                    return None;
                }
            }
        }
        Some(output_box)
    }
}
