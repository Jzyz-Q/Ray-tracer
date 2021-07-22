use crate::clamp;
use crate::vec3::*;
use crate::Perlin;
use image::GenericImageView;
use std::path::Path;
use std::sync::Arc;

pub trait Texture: Sync + Send {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3;
}

#[derive(Clone)]
pub struct CheckerT {
    pub odd: Arc<dyn Texture>,
    pub even: Arc<dyn Texture>,
}

impl CheckerT {
    /* pub fn new1(a: Arc<dyn Texture>, b: Arc<dyn Texture>) -> CheckerT {
        CheckerT { odd: a, even: b }
    }

    pub fn new(a: &Vec3, b: &Vec3) -> CheckerT {
        CheckerT {
            odd: Arc::new(Solid::new(*a)),
            even: Arc::new(Solid::new(*b)),
        }
    } */
}

impl Texture for CheckerT {
    // 3D棋盘格
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

#[derive(Copy, Clone)]
pub struct Solid {
    color: Vec3,
}

impl Solid {
    pub fn new(color: Vec3) -> Solid {
        Solid { color }
    }

    // pub fn new1(r: f64, g: f64, b: f64) -> Solid {
    //     Solid {
    //         color: Vec3::new(r, g, b),
    //     }
    // }
}

impl Texture for Solid {
    fn value(&self, _u: f64, _v: f64, _p: Vec3) -> Vec3 {
        self.color
    }
}

#[derive(Clone)]
pub struct Noise {
    noise: Perlin,
    scale: f64,
}

impl Noise {
    pub fn new(noise: Perlin, scale: f64) -> Noise {
        Noise { noise, scale }
    }
}

impl Texture for Noise {
    fn value(&self, _u: f64, _v: f64, p: Vec3) -> Vec3 {
        Vec3::ones() * 0.5 * (1.0 + (self.scale * p.z + 10.0 * self.noise.turb(p, 7)).sin())
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    pub img: image::DynamicImage,
    pub nx: u32,
    pub ny: u32,
}

impl ImageTexture {
    pub fn new(path: &Path) -> Self {
        let img = image::open(path).unwrap();
        Self {
            img: img.clone(),
            nx: img.dimensions().0,
            ny: img.dimensions().1,
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Vec3) -> Vec3 {
        let u = clamp(u, 0.0, 1.0);
        let v = 1.0 - clamp(v, 0.0, 1.0);
        let mut i = (u * self.nx as f64) as u32;
        let mut j = (v * self.ny as f64 - 0.001) as u32;

        if i >= self.nx {
            i = self.nx - 1;
        }
        if j >= self.ny {
            j = self.ny - 1;
        }
        let sc: f64 = 1.0 / 255.0;
        let pixel = self.img.get_pixel(i as u32, j as u32);
        Vec3::new(
            pixel[0] as f64 * sc,
            pixel[1] as f64 * sc,
            pixel[2] as f64 * sc,
        )
    }
}
