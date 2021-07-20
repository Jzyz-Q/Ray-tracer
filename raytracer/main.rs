mod camera;
mod hittable;
mod material;
mod ray;
mod vec3;
mod aabb;
mod bvh;
mod texture;
#[allow(clippy::float_cmp)]

use crate::camera::Camera;
use crate::hittable::Arc;
use crate::hittable::Hitrecord;
use crate::hittable::Hlist;
use crate::hittable::Object;
use crate::hittable::Sphere;
use crate::material::Dielectric;
use crate::material::Lambertian;
use crate::material::Metal;
use crate::ray::Ray;
use crate::vec3::Vec3;
use crate::texture::Solid;
use crate::texture::CheckerT;
use image::ImageBuffer;
use image::RgbImage;
use indicatif::ProgressBar;
use rand::rngs::ThreadRng;
use rand::Rng;

fn main() {
    let image_width = 1200;
    let image_height = 800;
    let spp = 500;
    let max_depth = 50;

    let mut img: RgbImage = ImageBuffer::new(image_width as u32, image_height as u32);
    let bar = ProgressBar::new(image_width as u64);

    println!("P3\n{0} {1}\n255\n", image_width, image_height);

    let w_f = image_width as f64;
    let h_f = image_height as f64;

    let aspect_ratio = w_f / h_f;
    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus: f64 = 10.0;
    let aperture = 0.0;

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    let world = two_sphere();

    // let m1 = Arc::<Lambertian>::new(Lambertian::new(Vec3::new(0.7, 0.3, 0.3)));
    // let m2 = Arc::<Lambertian>::new(Lambertian::new(Vec3::new(0.8, 0.8, 0.0)));
    // let m3 = Arc::<Metal>::new(Metal::new(Vec3::new(0.8, 0.6, 0.2), 0.5));
    // let m4 = Arc::<Metal>::new(Metal::new(Vec3::new(0.8, 0.8, 0.8), 0.5));
    // let m5 = Arc::<Dielectric>::new(Dielectric::new(1.5));

    // let foo1 = Arc::<Sphere>::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, m1.clone()));
    // let foo2 = Arc::<Sphere>::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, m2.clone()));
    // let foo3 = Arc::<Sphere>::new(Sphere::new(Vec3::new(1.0, 0.0, -1.0), 0.5, m3.clone()));
    // let foo4 = Arc::<Sphere>::new(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5, m4.clone()));
    // let foo5 = Arc::<Sphere>::new(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5, m5.clone()));

    // world.push(foo1);
    // world.push(foo2);
    // world.push(foo3);
    // world.push(foo4);
    // world.push(foo5);

    let mut _j = image_height - 1;
    while _j >= 0 {
        let mut _i = 0;
        while _i < image_width {
            let mut _s = 0;
            let i_f = _i as f64;
            let j_f = _j as f64;
            let mut color = Vec3::new(0.0, 0.0, 0.0);

            while _s < spp {
                let _u: f64 = (i_f + random_double()) / w_f;
                let _v: f64 = (j_f + random_double()) / h_f;
                let mut rng: ThreadRng = rand::thread_rng();
                let mut _r = cam.make_ray(&mut rng, _u, _v);
                //let r = Ray::new(cam.sor, cam.cor + cam.hor*u + cam.ver*v - cam.sor);
                color += ray_color(&_r, &world, max_depth);
                _s += 1;
            }
            let pixel = img.get_pixel_mut(_i as u32, _j as u32);
            let sppf = spp as f64;
            let scale: f64 = 1.0 / sppf;
            let mut _r = (color.x * scale).sqrt();
            let mut _g = (color.y * scale).sqrt();
            let mut _b = (color.z * scale).sqrt();

            let e1 = 256.0 * clamp(_r, 0.0, 0.999);
            let e2 = 256.0 * clamp(_g, 0.0, 0.999);
            let e3 = 256.0 * clamp(_b, 0.0, 0.999);

            let _a = e1 as i32;
            let _b = e2 as i32;
            let _c = e3 as i32;

            write_color(&color, spp);
            *pixel = image::Rgb([_a as u8, _b as u8, _c as u8]);
            _i += 1;
        }
        bar.inc(1);
        _j -= 1;
    }
    img.save("output/test.png").unwrap();
    bar.finish();
}

fn ray_color(_r: &Ray, world: &Hlist, depth: i32) -> Vec3 {
    let rec: Option<Hitrecord> = world.hit(&*_r, 0.001, std::f64::INFINITY);

    if depth <= 0 {
        let rt = Vec3::zero();
        return rt;
    }

    if let Some(val) = rec {
        let mut rng: ThreadRng = rand::thread_rng();
        //let mut target: Vec3 = val.p + val.n + random_unit_vector(&mut rng);
        //let mut tmp_r = Ray::new(val.p, target - val.p);
        //return ray_color(&tmp_r, &world, depth - 1) * 0.5;
        let cur = val.mat_ptr.scatter(&_r, &val, &mut rng);
        match cur {
            Some(scattered) => {
                let rt = Vec3::elemul(scattered.att, ray_color(&scattered.ray, &world, depth - 1));
                return rt;
            }
            None => {
                let rt = Vec3::new(0.0, 0.0, 0.0);
                return rt;
            }
        }
    }

    let unit_drc: Vec3 = _r.drc.unit();
    let _t: f64 = 0.5 * (unit_drc.y + 1.0);
    let tmp = Vec3::new(0.5, 0.7, 1.0);
    let one = Vec3::ones();
    one * (1.0 - _t) + tmp * _t
}

fn write_color(_s: &Vec3, spp: i32) {
    let sppf = spp as f64;
    let scale: f64 = 1.0 / sppf;
    let _r = (scale * _s.x).sqrt();
    let _g = (scale * _s.y).sqrt();
    let _b = (scale * _s.z).sqrt();

    let e1 = 256.0 * clamp(_r, 0.0, 0.999);
    let e2 = 256.0 * clamp(_g, 0.0, 0.999);
    let e3 = 256.0 * clamp(_b, 0.0, 0.999);

    let _a = e1 as i32;
    let _b = e2 as i32;
    let _c = e3 as i32;
    println!("{0} {1} {2}", _a, _b, _c);
}

fn clamp(_x: f64, _min: f64, _max: f64) -> f64 {
    if _x < _min {
        return _min;
    }
    if _x > _max {
        return _max;
    }
    _x
}

fn random_double() -> f64 {
    let rng: f64 = rand::thread_rng().gen();
    rng
}

fn random_double_limit(_min: f64, _max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(_min.._max)
}

fn random() -> Vec3 {
    Vec3::new(random_double(), random_double(), random_double())
}

fn random_limit(_min: f64, _max: f64) -> Vec3 {
    Vec3::new(
        random_double_limit(_min, _max),
        random_double_limit(_min, _max),
        random_double_limit(_min, _max),
    )
}

/* fn random_scene() -> Hlist {
    let mut world = Hlist::new(true);

    // let m1 = Arc::<Lambertian>::new(Lambertian::new(Vec3::new(0.7, 0.3, 0.3)));
    // let foo1 = Arc::<Sphere>::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, m1.clone()));
    world.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::<Lambertian>::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5))),
    )));

    let _i: i32 = 1;
    let _a: i32 = -11;
    let _b: i32 = -11;
    for _a in -11..11 {
        for _b in -11..11 {
            let choose_mat: f64 = random_double();
            let af = _a as f64;
            let bf = _b as f64;
            let center = Vec3::new(af + 0.9 * random_double(), 0.2, bf + 0.9 * random_double());
            let len = (center - Vec3::new(4.0, 0.2, 0.0)).length();
            if len > 0.9 {
                if choose_mat < 0.8 {
                    //disfuse
                    let albedo = Vec3::elemul(random(), random());
                    world.push(Arc::<Sphere>::new(Sphere::new(
                        center,
                        0.2,
                        Arc::<Lambertian>::new(Lambertian::new(albedo)),
                    )));
                } else if choose_mat < 0.95 {
                    //metal
                    let albedo = random_limit(0.5, 1.0);
                    let fuzz = random_double_limit(0.0, 0.5);
                    world.push(Arc::<Sphere>::new(Sphere::new(
                        center,
                        0.2,
                        Arc::<Metal>::new(Metal::new(albedo, fuzz)),
                    )));
                } else {
                    //glass
                    world.push(Arc::<Sphere>::new(Sphere::new(
                        center,
                        0.2,
                        Arc::<Dielectric>::new(Dielectric::new(1.5)),
                    )));
                }
            }
        }
    }

    world.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        Arc::<Dielectric>::new(Dielectric::new(1.5)),
    )));
    world.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        Arc::<Lambertian>::new(Lambertian::new(Vec3::new(0.4, 0.2, 0.1))),
    )));
    world.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        Arc::<Metal>::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0)),
    )));
    world
} */

fn two_sphere() -> Hlist {
    let mut objects = Hlist::new(true);

    let s1 = Arc::<Solid>::new(Solid::new(Vec3::new(0.2, 0.3, 0.1)));
    let s2 = Arc::<Solid>::new(Solid::new(Vec3::new(0.9, 0.9, 0.9)));
    let checker = Arc::<CheckerT>::new(CheckerT::new1(s1, s2));

    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, -10.0, 0.0),
        10.0,
        Arc::<Lambertian>::new(Lambertian::new(checker.clone()))
    )));

    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 10.0, 0.0),
        10.0,
        Arc::<Lambertian>::new(Lambertian::new(checker.clone()))
    )));
    objects
}
