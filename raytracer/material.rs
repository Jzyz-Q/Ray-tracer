use crate::{
    hittable::Hitrecord,
    ray::Ray,
    vec3::{random_unit_vector, Vec3},
};
use rand::rngs::ThreadRng;
use rand::Rng;
use crate::texture::Texture;
use std::sync::Arc;

#[derive(Copy, Clone)]
pub struct Scatter {
    pub att: Vec3, // 光线衰减率
    pub ray: Ray,
}

impl Scatter {
    pub fn new(att: Vec3, ray: Ray) -> Scatter {
        Scatter { att, ray }
    }
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    // 反射
    v - n * (2.0 * (v * n))
}

pub fn refract(uv: Vec3, n: Vec3, eoe: f64) -> Vec3 {
    // 折射   uv:入射光; n:法向量; eoe:折射率之比(η/η')
    let cos_t = (-uv) * n;
    let r_out_parallel: Vec3 = (uv + n * cos_t) * eoe;
    let r_out_perp: Vec3 = -n * (1.0 - r_out_parallel.squared_length()).sqrt();
    r_out_parallel + r_out_perp
}

pub trait Material: Send + Sync {
    fn scatter(&self, _r_in: &Ray, rec: &Hitrecord, _rng: &mut ThreadRng) -> Option<Scatter>;
    // fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color;
}

#[derive(Clone)]
pub struct Lambertian {
    // 漫反射材质
    pub albedo: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Arc<dyn Texture>) -> Lambertian {
        Lambertian { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &Hitrecord, _rng: &mut ThreadRng) -> Option<Scatter> {
        let s_drc: Vec3 = rec.n + random_unit_vector(_rng);
        let sed = Ray::new(rec.p, s_drc);
        let att = self.albedo.value(rec.u, rec.v, &rec.p);
        let rt = Scatter::new(att, sed);
        Some(rt)
    }
}

#[derive(Copy, Clone)]
pub struct Metal {
    // 金属材质 反射 粗糙
    pub albedo: Vec3,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(a: Vec3, fuzz: f64) -> Metal {
        Metal {
            albedo: Vec3 {
                x: a.x,
                y: a.y,
                z: a.z,
            },
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, _r_in: &Ray, rec: &Hitrecord, _rng: &mut ThreadRng) -> Option<Scatter> {
        let red: Vec3 = reflect(_r_in.drc.unit(), rec.n);
        let sed = Ray::new(
            rec.p,
            red + crate::vec3::random_in_unit_sphere(_rng) * self.fuzz,
        );
        let att = self.albedo;
        let rt = Scatter::new(att, sed);

        if sed.drc * rec.n > 0.0 {
            Some(rt)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)] // 玻璃 只发生折射
pub struct Dielectric {
    ref_idx: f64,
}

pub fn schlick(cosine: f64, ref_idx: f64) -> f64 {
    let r0_s = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0_s * r0_s;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

impl Dielectric {
    pub fn new(ref_idx: f64) -> Dielectric {
        Dielectric { ref_idx }
    }
}

impl Material for Dielectric {
    fn scatter(&self, _r_in: &Ray, rec: &Hitrecord, _rng: &mut ThreadRng) -> Option<Scatter> {
        let att = Vec3::new(1.0, 1.0, 1.0);
        let eoe = if rec.front_face {
            1.0 / self.ref_idx
        } else {
            self.ref_idx
        };

        let unit_drc: Vec3 = _r_in.drc.unit();
        let cos = -unit_drc * rec.n;
        let cos_theta: f64 = if cos < 1.0 { cos } else { 1.0 };
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        if eoe * sin_theta > 1.0 {
            // must reflect
            let reflected: Vec3 = reflect(unit_drc, rec.n);
            let sed = Ray::new(rec.p, reflected);
            let rt = Scatter::new(att, sed);
            return Some(rt);
        }

        let reflect_prob: f64 = schlick(cos_theta, eoe);
        let flag: f64 = rand::thread_rng().gen();
        if flag < reflect_prob {
            let reflected: Vec3 = reflect(unit_drc, rec.n);
            let sed = Ray::new(rec.p, reflected);
            let rt = Scatter::new(att, sed);
            return Some(rt);
        }

        let refracted = refract(unit_drc, rec.n, eoe);
        let sed = Ray::new(rec.p, refracted);
        let rt = Scatter::new(att, sed);
        Some(rt)
    }
}
