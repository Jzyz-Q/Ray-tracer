#![allow(warnings, unused)]

pub use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub org: Vec3,
    pub drc: Vec3,
    pub tm: f64,
}

impl Ray {
    pub fn new(org: Vec3, drc: Vec3, tm: f64) -> Self {
        Self { org, drc, tm }
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.org + self.drc * t
    }
}
