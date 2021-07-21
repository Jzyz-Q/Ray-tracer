#![allow(warnings, unused)]

use crate::{material::Material, ray::Ray, vec3::Vec3, aabb::AABB};
use std::ops::Mul;
pub use std::sync::Arc;
use core::f64::consts::PI;
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

#[derive(Clone)]
pub struct Xyrect {
    pub mat_ptr: Arc<dyn Material>,
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    pub k: f64, 
}

impl Xyrect {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, mat_ptr: Arc<dyn Material>) -> Xyrect {
        Xyrect {
            mat_ptr,
            x0,
            x1,
            y0,
            y1,
            k,
        }
    }
}

impl Object for Xyrect {
    fn hit(&self, ray: &Ray, t1_min: f64, t1_max: f64) -> Option<Hitrecord> {
        let t = (self.k - ray.org.z) / ray.drc.z;
        if t < t1_min || t > t1_max {
            return None;
        }
        let x = ray.org.x + t * ray.drc.x;
        let y = ray.org.y + t * ray.drc.y;
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let mut outward_normal = Vec3::new(0.0, 0.0, 1.0);
        let flag = (ray.drc * outward_normal) < 0.0;
        if !flag {
            outward_normal = -outward_normal;
        }
        Some(Hitrecord {
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (y - self.y0) / (self.y1 - self.y0),
            t,
            n: outward_normal,
            front_face: flag,
            mat_ptr: self.mat_ptr.clone(),
            p: ray.at(t),
        })
    }
    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        Some(AABB::new(
            &Vec3::new(self.x0, self.y0, self.k - 0.0001),
            &Vec3::new(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}

#[derive(Clone)]
pub struct Xzrect {
    pub mat_ptr: Arc<dyn Material>,
    pub x0: f64,
    pub x1: f64,
    pub z0: f64,
    pub z1: f64,
    pub k: f64, 
}

impl Xzrect {
    pub fn new(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, mat_ptr: Arc<dyn Material>) -> Xzrect {
        Xzrect {
            mat_ptr,
            x0,
            x1,
            z0,
            z1,
            k,
        }
    }
}

impl Object for Xzrect {
    fn hit(&self, ray: &Ray, t1_min: f64, t1_max: f64) -> Option<Hitrecord> {
        let t = (self.k - ray.org.y) / ray.drc.y;
        if t < t1_min || t > t1_max {
            return None;
        }
        let x = ray.org.x + t * ray.drc.x;
        let z = ray.org.z + t * ray.drc.z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }
        let mut outward_normal = Vec3::new(0.0, 1.0, 0.0);
        let flag = (ray.drc * outward_normal) < 0.0;
        if !flag {
            outward_normal = -outward_normal;
        }
        Some(Hitrecord {
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (z - self.z0) / (self.z1 - self.z0),
            t,
            n: outward_normal,
            front_face: flag,
            mat_ptr: self.mat_ptr.clone(),
            p: ray.at(t),
        })
    }
    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        Some(AABB::new(
            &Vec3::new(self.x0, self.k - 0.0001, self.z0),
            &Vec3::new(self.x1, self.k + 0.0001, self.z1),
        ))
    }
}

#[derive(Clone)]
pub struct Yzrect {
    pub mat_ptr: Arc<dyn Material>,
    pub y0: f64,
    pub y1: f64,
    pub z0: f64,
    pub z1: f64,
    pub k: f64, 
}

impl Yzrect {
    pub fn new(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, mat_ptr: Arc<dyn Material>) -> Yzrect {
        Yzrect {
            mat_ptr,
            y0,
            y1,
            z0,
            z1,
            k,
        }
    }
}

impl Object for Yzrect {
    fn hit(&self, ray: &Ray, t1_min: f64, t1_max: f64) -> Option<Hitrecord> {
        let t = (self.k - ray.org.x) / ray.drc.x;
        if t < t1_min || t > t1_max {
            return None;
        }
        let y = ray.org.y + t * ray.drc.y;
        let z = ray.org.z + t * ray.drc.z;
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }
        let mut outward_normal = Vec3::new(1.0, 0.0, 0.0);
        let flag = (ray.drc * outward_normal) < 0.0;
        if !flag {
            outward_normal = -outward_normal;
        }
        Some(Hitrecord {
            u: (y - self.y0) / (self.y1 - self.y0),
            v: (z - self.z0) / (self.z1 - self.z0),
            t,
            n: outward_normal,
            front_face: flag,
            mat_ptr: self.mat_ptr.clone(),
            p: ray.at(t),
        })
    }
    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        Some(AABB::new(
            &Vec3::new(self.k - 0.0001, self.y0, self.z0),
            &Vec3::new(self.k + 0.0001, self.y1, self.z1),
        ))
    }
}

#[derive(Clone)]
pub struct Boxes {
    pub box_min: Vec3,
    pub box_max: Vec3,
    pub sides: (
        Xyrect,
        Xyrect,
        Xzrect,
        Xzrect,
        Yzrect,
        Yzrect,
    ),
}

impl Boxes {
    pub fn new(p0: &Vec3, p1: &Vec3, mat_ptr: Arc<dyn Material>) -> Boxes {
        Boxes {
            box_min: *p0,
            box_max: *p1,
            sides: (
                Xyrect {
                    x0: p0.x,
                    x1: p1.x,
                    y0: p0.y,
                    y1: p1.y,
                    k: p1.z,
                    mat_ptr: mat_ptr.clone()
                },
                Xyrect {
                    x0: p0.x,
                    x1: p1.x,
                    y0: p0.y,
                    y1: p1.y,
                    k: p0.z,
                    mat_ptr: mat_ptr.clone()
                },
                Xzrect {
                    x0: p0.x,
                    x1: p1.x,
                    z0: p0.z,
                    z1: p1.z,
                    k: p1.y,
                    mat_ptr: mat_ptr.clone()
                },
                Xzrect {
                    x0: p0.x,
                    x1: p1.x,
                    z0: p0.z,
                    z1: p1.z,
                    k: p0.y,
                    mat_ptr: mat_ptr.clone()
                },
                Yzrect {
                    y0: p0.y,
                    y1: p1.y,
                    z0: p0.z,
                    z1: p1.z,
                    k: p0.x,
                    mat_ptr: mat_ptr.clone()
                },
                Yzrect {
                    y0: p0.y,
                    y1: p1.y,
                    z0: p0.z,
                    z1: p1.z,
                    k: p1.x,
                    mat_ptr: mat_ptr.clone()
                },
            ),
        }
    }
}

impl Object for Boxes {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Hitrecord> {
        let mut result: Option<Hitrecord> = None;
        let mut closest = t_max;
        if let Some(rec) = self.sides.0.hit(ray, t_min, closest) {
            closest = rec.t;
            result = Some(rec);
        }
        if let Some(rec) = self.sides.1.hit(ray, t_min, closest) {
            closest = rec.t;
            result = Some(rec);
        }
        if let Some(rec) = self.sides.2.hit(ray, t_min, closest) {
            closest = rec.t;
            result = Some(rec);
        }
        if let Some(rec) = self.sides.3.hit(ray, t_min, closest) {
            closest = rec.t;
            result = Some(rec);
        }
        if let Some(rec) = self.sides.4.hit(ray, t_min, closest) {
            closest = rec.t;
            result = Some(rec);
        }
        if let Some(rec) = self.sides.5.hit(ray, t_min, closest) {
            result = Some(rec);
        }
        result
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        Some(AABB::new(&self.box_min, &self.box_max))
    }
}

#[derive(Clone)]
pub struct Translate {
    pub ptr: Arc<dyn Object>,
    pub offset: Vec3,
}

impl Translate {
    pub fn new(p: Arc<dyn Object>, displacement: &Vec3) -> Self {
        Self {
            ptr: p,
            offset: *displacement,
        }
    }
}

impl Object for Translate {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Hitrecord> {
        let moved_r = Ray::new(ray.org - self.offset, ray.drc);
        if let Some(mut rec) = self.ptr.hit(&moved_r, t_min, t_max) {
            let flag = (moved_r.drc * rec.n) < 0.0;
            if !flag {
                rec.n = -rec.n;
            }
            return Some(Hitrecord {
                p: rec.p + self.offset,
                n: rec.n,
                front_face: flag,
                mat_ptr: rec.mat_ptr,
                u: rec.u,
                t: rec.t,
                v: rec.v,
            });
        }
        None
    }

    fn bounding_box(&self, t0: f64, t1: f64) -> Option<AABB> {
        if let Some(output) = self.ptr.bounding_box(t0, t1) {
            return Some(AABB::new(
                &(output._min + self.offset),
                &(output._max + self.offset),
            ));
        }
        None
    }
}

#[derive(Clone)]
pub struct RotateY {
    pub ptr: Arc<dyn Object>,
    pub sin_theta: f64,
    pub cos_theta: f64,
    pub has_box: bool,
    pub bbox: AABB,
}

impl RotateY {
    pub fn new(ptr: Arc<dyn Object>, angle: f64) -> Self {
        let radians = angle / 180.0 * PI;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let mut _min = Vec3::new(std::f64::INFINITY, std::f64::INFINITY, std::f64::INFINITY);
        let mut _max = Vec3::new(-std::f64::INFINITY, -std::f64::INFINITY, -std::f64::INFINITY);

        if let Some(bbox) = ptr.bounding_box(0.0, 1.0) {
            let has_box = true;
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x = i as f64 * bbox._max.x + (1 - i) as f64 * bbox._min.x;
                        let y = j as f64 * bbox._max.y + (1 - j) as f64 * bbox._min.y;
                        let z = k as f64 * bbox._max.z + (1 - k) as f64 * bbox._min.z;

                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        _min.x = _min.x.min(newx);
                        _min.y = _min.y.min(y);
                        _min.z = _min.z.min(newz);
                        _max.x = _max.x.max(newx);
                        _max.y = _max.y.max(y);
                        _max.z = _max.z.max(newz);
                    }
                }
            }
            let bbox = AABB::new(&_min, &_max);
            Self {
                ptr,
                sin_theta,
                cos_theta,
                bbox,
                has_box,
            }
        } else {
            panic!();
        }
    }
}

impl Object for RotateY {
    
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Hitrecord> {
        let mut org = ray.org;
        let mut drc = ray.drc;

        org.x = self.cos_theta * ray.org.x - self.sin_theta * ray.org.z;
        org.z = self.sin_theta * ray.org.x + self.cos_theta * ray.org.z;

        drc.x = self.cos_theta * ray.drc.x - self.sin_theta * ray.drc.z;
        drc.z = self.sin_theta * ray.drc.x + self.cos_theta * ray.drc.z;

        let rotated_r = Ray::new(org, drc);
        if let Some(rec) = self.ptr.hit(&rotated_r, t_min, t_max) {
            let mut p = rec.p;
            let mut n = rec.n;

            p.x = self.cos_theta * rec.p.x + self.sin_theta * rec.p.z;
            p.z = -self.sin_theta * rec.p.x + self.cos_theta * rec.p.z;

            n.x = self.cos_theta * rec.n.x + self.sin_theta * rec.n.z;
            n.z = -self.sin_theta * rec.n.x + self.cos_theta * rec.n.z;

            let flag = (rotated_r.drc * rec.n) < 0.0;
            if !flag {
                n = -n;
            }
            return Some(Hitrecord {
                p,
                n,
                front_face: flag,
                mat_ptr: rec.mat_ptr,
                u: rec.u,
                t: rec.t,
                v: rec.v,
            });
        }
        None
    }

    fn bounding_box(&self, _t0: f64, _t1: f64) -> Option<AABB> {
        if self.has_box {
            Some(self.bbox)
        } else {
            None
        }
    }
}