use crate::bvh::random_in_unit_sphere;
use crate::ThreadRng;
use crate::Vec3;
use rand::random;

pub const POINT_COUNT: usize = 256;

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
        for _i in 0..POINT_COUNT {
            //ranvec.push(random_limit(-1.0, 1.0).unit());
            let mut rng: ThreadRng = rand::thread_rng();
            ranvec.push(random_in_unit_sphere(&mut rng));
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

    pub fn noise(&self, p: Vec3) -> f64 {
        let _u = p.x - p.x.floor();
        let _v = p.y - p.y.floor();
        let _w = p.z - p.z.floor();

        let uu = _u * _u * (3.0 - 2.0 * _u);
        let vv = _v * _v * (3.0 - 2.0 * _v);
        let ww = _w * _w * (3.0 - 2.0 * _w);

        let _i = p.x.floor() as i32;
        let _j = p.y.floor() as i32;
        let _k = p.z.floor() as i32;
        let mut c: [[[Vec3; 2]; 2]; 2] = [[[Vec3::zero(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranvec[(self.perm_x[255 & (_i + di as i32) as usize]
                        ^ self.perm_y[255 & (_j + dj as i32) as usize]
                        ^ self.perm_z[255 & (_k + dk as i32) as usize])
                        as usize];
                }
            }
        }
        interp(c, uu, vv, ww)
        //self.ranfloat[self.perm_x[_i] ^ self.perm_y[_j] ^ self.perm_z[_k]]
    }

    pub fn turb(&self, p: Vec3, depth: i32) -> f64 {
        //depth == 7
        let mut accum: f64 = 0.0;
        let mut temp_p: Vec3 = p;
        let mut weight: f64 = 1.0;

        for _i in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }
}

pub fn perlin_generate_perm() -> Vec<usize> {
    let mut p: Vec<usize> = Vec::new();

    for i in 0..POINT_COUNT {
        p.push(i as usize);
    }
    permute(&mut p, POINT_COUNT as i32);
    p
}

pub fn permute(p: &mut Vec<usize>, n: i32) {
    for i in (0..n).rev() {
        let i = i as usize;
        //let target = random_int(0, n-i);
        let target = random::<usize>() % (i + 1);
        (*p).swap(i as usize, target as usize)
        // let tmp = p[n-i];
        // p[n-i] = p[target];
        // p[target] = tmp;
    }
}

pub fn interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
    let mut accum: f64 = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let weight_v = Vec3::new(u - (i as f64), v - (j as f64), w - (k as f64));
                accum += ((i as f64) * u + (1.0 - (i as f64)) * (1.0 - u))
                    * ((j as f64) * v + (1.0 - (j as f64)) * (1.0 - v))
                    * ((k as f64) * w + (1.0 - (k as f64)) * (1.0 - w))
                    * (c[i][j][k] * weight_v);
            }
        }
    }
    accum
}
