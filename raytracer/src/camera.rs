#![allow(warnings, unused)]
use crate::random_double_limit;
use crate::{ray::Ray, vec3::random_in_unit_disk, vec3::Vec3};
use rand::rngs::ThreadRng;

// 对一个像素进行多次采样
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub sor: Vec3,
    pub cor: Vec3,
    pub hor: Vec3,
    pub ver: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub len_r: f64,
    pub time0: f64,
    pub time1: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Vec3,
        lookat: Vec3,
        vup: Vec3,
        vfov: f64,
        aspect: f64,
        aperture: f64,
        focus_dist: f64,
        t0: f64,
        t1: f64,
    ) -> Camera {
        let theta = vfov * std::f64::consts::PI / 180.0;
        let h_height: f64 = (theta / 2.0).tan();
        let h_width: f64 = aspect * h_height;

        let w1 = (lookfrom - lookat).unit();
        let u1 = (Vec3::cross(vup, w1)).unit();
        let v1 = Vec3::cross(w1, u1);

        Camera {
            sor: lookfrom,
            w: w1,
            u: u1,
            v: v1,
            cor: lookfrom
                - u1 * h_width * focus_dist
                - v1 * h_height * focus_dist
                - w1 * focus_dist,
            hor: u1 * 2.0 * h_width * focus_dist,
            ver: v1 * 2.0 * h_height * focus_dist,
            len_r: 0.5 * aperture,
            time0: t0,
            time1: t1,
        }
    }

    pub fn make_ray(&self, rng: &mut ThreadRng, a: f64, b: f64) -> Ray {
        let rd: Vec3 = random_in_unit_disk(rng) * self.len_r;
        let offset: Vec3 = self.u * rd.x + self.v * rd.y;

        let rt = Ray::new(
            self.sor + offset,
            self.cor + self.hor * a + self.ver * b - self.sor - offset,
            random_double_limit(self.time0, self.time1),
        );
        return rt;
    }
}
