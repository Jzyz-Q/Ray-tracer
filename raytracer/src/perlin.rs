use rand::Rng;
use crate::Vec3;
use crate::random_double_limit;
use crate::random_limit;

#[derive(Clone)]
pub struct Perlin {
    ranvec: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    pub fn new() -> Perlin {
        let mut ranvec: Vec<Vec3> = Vec::new();
        for i in 0..256 {
            ranvec.push(random_limit(-1.0, 1.0).unit());
        }

        let perm_x = perlin_generate_perm();
        let perm_y = perlin_generate_perm();
        let perm_z = perlin_generate_perm();

        Perlin {
            ranvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: &Vec3) -> f64 {
        let _u = p.x - p.x.floor();
        let _v = p.y - p.y.floor();
        let _w = p.z - p.z.floor();

        let _u = _u*_u*(3.0-2.0*_u);
        let _v = _v*_v*(3.0-2.0*_v);
        let _w = _w*_w*(3.0-2.0*_w);

        let _i = (255 & (p.x.floor() as i32)) as usize;
        let _j = (255 & (p.x.floor() as i32)) as usize;
        let _k = (255 & (p.x.floor() as i32)) as usize;
        let mut c: [[[Vec3; 2]; 2]; 2] = [[[Vec3::zero(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranvec[
                        self.perm_x[255 & (_i+di)] ^
                        self.perm_y[255 & (_j+dj)] ^
                        self.perm_z[255 & (_k+dk)]
                    ];
                }
            }
        }
        interp(c, _u, _v, _w)
        //self.ranfloat[self.perm_x[_i] ^ self.perm_y[_j] ^ self.perm_z[_k]]
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

pub fn interp(mut c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
    let mut accum: f64 = 0.0;
    for i in 0..2{
        for j in 0..2 {
            for k in 0..2 {
                let weight_v = Vec3::new(u-(i as f64), v-(j as f64), w-(k as f64));
                accum += ((i as f64)*u + (1.0 - (i as f64))*(1.0 - u)) *
                         ((j as f64)*v + (1.0 - (j as f64))*(1.0 - v)) *
                         ((k as f64)*w + (1.0 - (k as f64))*(1.0 - w)) *
                         (c[i][j][k]*weight_v);
            }
        }
    }
    accum
}
