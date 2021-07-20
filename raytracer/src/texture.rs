use crate::vec3::*;
use std::sync::Arc;
use crate::Perlin;

pub trait Texture: Sync + Send {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3;
} 

#[derive(Clone)]
pub struct CheckerT {
    pub odd: Arc<dyn Texture>,
    pub even: Arc<dyn Texture>,
}

impl CheckerT {
    pub fn new1(a: Arc<dyn Texture>, b: Arc<dyn Texture>) -> CheckerT {
        CheckerT { odd: a, even: b }
    }

    pub fn new(a: &Vec3, b: &Vec3) -> CheckerT {
        CheckerT {
            odd: Arc::new(Solid::new(*a)),
            even: Arc::new(Solid::new(*b)),
        }
    } 
}

impl Texture for CheckerT {       // 3D棋盘格
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();
        if sines < 0.0 {self.odd.value(u, v, p)}
        else {return self.even.value(u, v, p);}
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

    pub fn new1(r: f64, g: f64, b: f64) -> Solid {
        Solid {
            color: Vec3::new(r, g, b)
        }
    }
}

impl Texture for Solid {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
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
    fn value(&self, u: f64, v: f64, p: Vec3) -> Vec3 {
        Vec3::ones() * 0.5 * (1.0 + self.scale * p.z + 10.0 * self.noise.turb(p, 7)).sin()
    }
}
