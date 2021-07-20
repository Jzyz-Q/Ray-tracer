use rand::Rng;
use crate::Vec3;
use crate::random_double;

#[derive(Clone)]
pub struct Perlin {
    ranfloat: Vec<f64>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    pub fn new() -> Perlin {
        let mut ranfloat: Vec<f64> = Vec::new();
        for i in 0..256 {
            ranfloat.push(random_double());
        }

        let perm_x = perlin_generate_perm();
        let perm_y = perlin_generate_perm();
        let perm_z = perlin_generate_perm();

        Perlin {
            ranfloat,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: &Vec3) -> f64 {
        let _u = p.x - p.x.floor();
        let _v = p.y - p.y.floor();
        let _w = p.z - p.z.floor();

        let _i = (255 & ((4.0 * p.x) as i32)) as usize;
        let _j = (255 & ((4.0 * p.y) as i32)) as usize;
        let _k = (255 & ((4.0 * p.z) as i32)) as usize;

        self.ranfloat[self.perm_x[_i] ^ self.perm_y[_j] ^ self.perm_z[_k]]
    }
}

pub fn perlin_generate_perm() -> Vec<usize> {
    let mut p: Vec<usize> = Vec::new();

    for i in 0..256 {
        p.push(i);
    }
    permute(p.clone(), 256);
    return p.clone();
}

pub fn permute(mut p: Vec<usize>, n: usize) {
    for i in 1..n {
        let target = random_int(0, n-i);
        let tmp = p[n-i];
        p[n-i] = p[target];
        p[target] = tmp;
    }
}

pub fn random_int(a: usize, b: usize) -> usize {
    let mut rng = rand::thread_rng();
    return rng.gen_range(a..b);
}