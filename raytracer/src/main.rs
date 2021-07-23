#[allow(clippy::float_cmp)]
mod aabb;
mod bvh;
mod camera;
mod hittable;
mod material;
// mod perlin;
mod ray;
mod texture;
mod vec3;
use crate::bvh::Boxes;
use crate::bvh::BvhNode;
use crate::bvh::ConstantMedium;
// use crate::bvh::RotateY;
// use crate::bvh::Translate;
use crate::bvh::Xyrect;
use crate::bvh::Xzrect;
use crate::bvh::Yzrect;
use crate::camera::Camera;
use crate::hittable::Arc;
use crate::hittable::Hitrecord;
use crate::hittable::Hlist;
use crate::hittable::Object;
use crate::hittable::Sphere;
use crate::material::Dielectric;
use crate::material::Diffuse;
use crate::material::Lambertian;
use crate::material::Metal;
// use crate::perlin::Perlin;
use crate::ray::Ray;
// use crate::texture::CheckerT;
use crate::hittable::MovingSphere;
use crate::texture::ImageTexture;
// use crate::texture::Noise;
// use crate::material::Isotropic;
use crate::texture::Solid;
use crate::vec3::Vec3;
use image::ImageBuffer;
use image::RgbImage;
use indicatif::ProgressBar;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

fn main() {
    let is_ci = match std::env::var("CI") {
        Ok(x) => x == "true",
        Err(_) => false,
    };

    let (n_jobs, n_workers): (usize, usize) = if is_ci { (32, 2) } else { (16, 2) };

    println!(
        "CI: {}, using {} jobs and {} workers",
        is_ci, n_jobs, n_workers
    );

    let width = 500;
    let height = 500;
    let spp = 1000;
    let max_depth = 50;
    let background = Vec3::zero();

    let (tx, rx) = channel();
    let pool = ThreadPool::new(n_workers);

    //let mut img: RgbImage = ImageBuffer::new(image_width as u32, image_height as u32);
    let bar = ProgressBar::new(n_jobs as u64);

    println!("P3\n{0} {1}\n255\n", width, height);

    let w_f = width as f64;
    let h_f = height as f64;

    let aspect_ratio = w_f / h_f;
    let lookfrom = Vec3::new(278.0, 278.0, -780.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus: f64 = 10.0;
    let aperture = 0.0;

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        40.0,
        aspect_ratio,
        aperture,
        dist_to_focus,
        0.0,
        1.0,
    );

    let world = cloud();

    // let mut _j = image_height - 1;
    // while _j >= 0 {
    //     let mut _i = 0;
    //     while _i < image_width {
    //         let mut _s = 0;
    //         let i_f = _i as f64;
    //         let j_f = _j as f64;
    //         let mut color = Vec3::new(0.0, 0.0, 0.0);

    //         while _s < spp {
    //             let _u: f64 = (i_f + random_double()) / w_f;
    //             let _v: f64 = (j_f + random_double()) / h_f;
    //             let mut rng: ThreadRng = rand::thread_rng();
    //             let mut _r = cam.make_ray(&mut rng, _u, _v);
    //             //let r = Ray::new(cam.sor, cam.cor + cam.hor*u + cam.ver*v - cam.sor);
    //             color += ray_color(&_r, &background, &world, max_depth);
    //             _s += 1;
    //         }
    //         let pixel = img.get_pixel_mut(_i as u32, _j as u32);
    //         let sppf = spp as f64;
    //         let scale: f64 = 1.0 / sppf;
    //         let mut _r = (color.x * scale).sqrt();
    //         let mut _g = (color.y * scale).sqrt();
    //         let mut _b = (color.z * scale).sqrt();

    //         let e1 = 256.0 * clamp(_r, 0.0, 0.999);
    //         let e2 = 256.0 * clamp(_g, 0.0, 0.999);
    //         let e3 = 256.0 * clamp(_b, 0.0, 0.999);

    //         let _a = e1 as i32;
    //         let _b = e2 as i32;
    //         let _c = e3 as i32;

    //         write_color(&color, spp);
    //         *pixel = image::Rgb([_a as u8, _b as u8, _c as u8]);
    //         _i += 1;
    //     }
    //     bar.inc(1);
    //     _j -= 1;
    // }

    for i in 0..n_jobs {
        let tx = tx.clone();
        let world_ptr = world.clone();
        pool.execute(move || {
            let row_begin = height as usize * i / n_jobs;
            let row_end = height as usize * (i + 1) / n_jobs;
            let render_height = row_end - row_begin;
            let mut img: RgbImage = ImageBuffer::new(width, render_height as u32);
            for x in 0..width {
                for (img_y, y) in (row_begin..row_end).enumerate() {
                    let y = y as u32;
                    let mut _s = 0;
                    let mut color = Vec3::new(0.0, 0.0, 0.0);
                    while _s < spp {
                        let _u: f64 = (x as f64 + random_double()) / w_f;
                        let _v: f64 = (y as f64 + random_double()) / h_f;
                        let mut rng: ThreadRng = rand::thread_rng();
                        let mut _r = cam.make_ray(&mut rng, _u, _v);
                        color += ray_color(&_r, &background, &world_ptr, max_depth);
                        _s += 1;
                    }
                    let pixel = img.get_pixel_mut(x, img_y as u32);
                    //let color = world_ptr.color(x, y);
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
                }
            }
            tx.send((row_begin..row_end, img))
                .expect("failed to send result");
        });
    }

    let mut img: RgbImage = ImageBuffer::new(width, height);

    for (rows, data) in rx.iter().take(n_jobs) {
        for (idx, row) in rows.enumerate() {
            for col in 0..width {
                let row = row as u32;
                let idx = idx as u32;
                *img.get_pixel_mut(col, row) = *data.get_pixel(col, idx);
            }
        }
        bar.inc(1);
    }

    img.save("output/test.png").unwrap();
    bar.finish();
}

fn ray_color(_r: &Ray, background: &Vec3, world: &Hlist, depth: i32) -> Vec3 {
    let rec: Option<Hitrecord> = world.hit(&*_r, 0.001, std::f64::INFINITY);

    if depth <= 0 {
        let rt = Vec3::zero();
        return rt;
    }

    match rec {
        Some(val) => {
            let emitted = val.mat_ptr.emitted(val.u, val.v, &val.p);
            let mut rng: ThreadRng = rand::thread_rng();
            let cur = val.mat_ptr.scatter(&_r, &val, &mut rng);
            match cur {
                Some(scattered) => {
                    Vec3::elemul(
                        scattered.att,
                        ray_color(&scattered.ray, &background, &world, depth - 1),
                    ) + emitted
                }
                None => emitted,
            }
        }
        None => *background,
    }

    // let unit_drc: Vec3 = _r.drc.unit();
    // let _t: f64 = 0.5 * (unit_drc.y + 1.0);
    // let tmp = Vec3::new(0.5, 0.7, 1.0);
    // let one = Vec3::ones();
    // one * (1.0 - _t) + tmp * _t
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

/* fn random_limit(_min: f64, _max: f64) -> Vec3 {
    Vec3::new(
        random_double_limit(_min, _max),
        random_double_limit(_min, _max),
        random_double_limit(_min, _max),
    )
}
 */
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

/* fn two_sphere() -> Hlist {
    let mut objects = Hlist::new(true);

    let s1 = Arc::<Solid>::new(Solid::new(Vec3::new(0.2, 0.3, 0.1)));
    let s2 = Arc::<Solid>::new(Solid::new(Vec3::new(0.9, 0.9, 0.9)));
    let checker = Arc::<CheckerT>::new(CheckerT::new1(s1, s2));

    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, -10.0, 0.0),
        10.0,
        Arc::<Lambertian>::new(Lambertian::new(checker.clone())),
    )));

    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 10.0, 0.0),
        10.0,
        Arc::<Lambertian>::new(Lambertian::new(checker.clone())),
    )));
    objects
} */

/* fn tow_perlin_spheres() -> Hlist {
    let mut objects = Hlist::new(true);

    let pn = Perlin::new();
    let pertext = Arc::<Noise>::new(Noise::new(pn, 4.0));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::<Lambertian>::new(Lambertian::new(pertext.clone())),
    )));

    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::<Lambertian>::new(Lambertian::new(pertext.clone())),
    )));
    objects
} */

/* fn earth() -> Hlist {
    let path = Path::new("input.jpg");
    let mut objects = Hlist::new(true);

    let imgtext = Arc::<ImageTexture>::new(ImageTexture::new(path));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        2.0,
        Arc::<Lambertian>::new(Lambertian::new(imgtext.clone())),
    )));
    objects
} */

/* fn simple_light() -> Hlist {
    let mut objects = Hlist::new(true);

    let pn = Perlin::new();
    let pertext = Arc::<Noise>::new(Noise::new(pn, 4.0));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::<Lambertian>::new(Lambertian::new(pertext.clone())),
    )));

    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::<Lambertian>::new(Lambertian::new(pertext.clone())),
    )));

    let s1 = Arc::<Solid>::new(Solid::new(Vec3::new(4.0, 4.0, 4.0)));
    let difflight = Arc::<Diffuse>::new(Diffuse::new(s1));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 7.0, 0.0),
        2.0,
        difflight.clone(),
    )));

    objects.push(Arc::<Xyrect>::new(Xyrect::new(
        3.0,
        5.0,
        1.0,
        3.0,
        -2.0,
        difflight.clone(),
    )));
    objects
} */

/* fn cornell_box() -> Hlist {
    let mut objects = Hlist::new(true);

    let s1 = Arc::<Solid>::new(Solid::new(Vec3::new(7.0, 7.0, 7.0)));
    let vr = Arc::<Solid>::new(Solid::new(Vec3::new(0.65, 0.05, 0.05)));
    let vw = Arc::<Solid>::new(Solid::new(Vec3::new(0.73, 0.73, 0.73)));
    let vg = Arc::<Solid>::new(Solid::new(Vec3::new(0.12, 0.45, 0.15)));
    let v1 = Arc::<Solid>::new(Solid::new(Vec3::new(0.0, 0.0, 0.0)));
    let v2 = Arc::<Solid>::new(Solid::new(Vec3::new(1.0, 1.0, 1.0)));

    let red = Arc::<Lambertian>::new(Lambertian::new(vr));
    let white = Arc::<Lambertian>::new(Lambertian::new(vw));
    let green = Arc::<Lambertian>::new(Lambertian::new(vg));
    let light = Arc::<Diffuse>::new(Diffuse::new(s1));

    let box1 = Arc::<Boxes>::new(Boxes::new(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 330.0, 165.0),
        white.clone(),
    ));
    let box1 = Arc::<RotateY>::new(RotateY::new(box1.clone(), 15.0));
    let box1 = Arc::<Translate>::new(Translate::new(box1.clone(), &Vec3::new(265.0, 0.0, 265.0)));
    let box1 = Arc::<ConstantMedium>::new(ConstantMedium::new(box1.clone(), 0.01, v1));
    objects.push(box1);

    let box2 = Arc::<Boxes>::new(Boxes::new(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(165.0, 165.0, 165.0),
        white.clone(),
    ));
    let box2 = Arc::<RotateY>::new(RotateY::new(box2.clone(), -18.0));
    let box2 = Arc::<Translate>::new(Translate::new(box2.clone(), &Vec3::new(130.0, 0.0, 65.0)));
    let box2 = Arc::<ConstantMedium>::new(ConstantMedium::new(box2.clone(), 0.01, v2));
    objects.push(box2);

    objects.push(Arc::<Yzrect>::new(Yzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        green.clone(),
    )));

    objects.push(Arc::<Yzrect>::new(Yzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        red.clone(),
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        113.0,
        443.0,
        127.0,
        432.0,
        554.0,
        light.clone(),
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        white.clone(),
    )));

    objects.push(Arc::<Xyrect>::new(Xyrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    )));
    objects
} */

/* fn final_scene() -> Hlist {
    let mut objects = Hlist::new(true);
    let mut boxes1 = Hlist::new(true);

    // light
    let vl = Arc::<Solid>::new(Solid::new(Vec3::new(20.0, 20.0, 20.0)));
    let light = Arc::<Diffuse>::new(Diffuse::new(vl));
    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        203.0, 343.0, 227.0, 332.0, 554.0, light,
    )));

    // background && ground
    let s1 = Arc::<Solid>::new(Solid::new(Vec3::new(0.498, 0.533, 0.796)));
    let ground = Arc::<Lambertian>::new(Lambertian::new(s1));

    let vb = Arc::<Solid>::new(Solid::new(Vec3::new(0.184, 0.2157, 0.4588)));
    let vy = Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.886)));

    let blue = Arc::<Lambertian>::new(Lambertian::new(vb));
    let yellow = Arc::<Lambertian>::new(Lambertian::new(vy.clone()));

    // a box on the ground
    let box2 = Arc::<Boxes>::new(Boxes::new(
        &Vec3::new(0.0, 0.0, 0.0),
        &Vec3::new(125.0, 125.0, 125.0),
        yellow,
    ));
    let box2 = Arc::<RotateY>::new(RotateY::new(box2, -18.0));
    let box2 = Arc::<Translate>::new(Translate::new(box2, &Vec3::new(130.0, 0.0, 65.0)));
    let box2 = Arc::<ConstantMedium>::new(ConstantMedium::new(box2, 0.01, vy));
    objects.push(box2);

    // background picture moon
    let path = Path::new("moon.jpg");
    let imgtext = Arc::<ImageTexture>::new(ImageTexture::new(path));
    let moon = Arc::<Lambertian>::new(Lambertian::new(imgtext));

    // sky
    let path_sky = Path::new("sky.jpg");
    let imgtext = Arc::<ImageTexture>::new(ImageTexture::new(path_sky));
    let sky = Arc::<Lambertian>::new(Lambertian::new(imgtext));

    objects.push(Arc::<Yzrect>::new(Yzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        blue.clone(),
    )));

    objects.push(Arc::<Yzrect>::new(Yzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        blue.clone(),
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        0.0, 555.0, 0.0, 555.0, 0.0, blue,
    )));

    objects.push(Arc::<Xyrect>::new(Xyrect::new(
        //背景
        0.0, 555.0, 0.0, 555.0, 555.0, moon,
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        0.0, 555.0, 0.0, 555.0, 555.0, sky,
    )));

    let boxex_per_side = 20;
    for _i in 0..boxex_per_side {
        for _j in 0..boxex_per_side {
            let w: f64 = 100.0;
            let x0: f64 = -1000.0 + w * _i as f64;
            let z0: f64 = -1000.0 + w * _j as f64;
            let y0: f64 = 0.0;
            let x1: f64 = x0 + w;
            let y1: f64 = random_double_limit(1.0, 101.0);
            let z1: f64 = z0 + w;

            /* let v1 = Arc::<Solid>::new(Solid::new(Vec3::new(0.9568, 0.694, 0.5137)));

            let box1 = Arc::<Boxes>::new(Boxes::new(
                &Vec3::new(x0, y0, z0),
                &Vec3::new(x1, y1, z1),
                ground.clone(),
            ));
            let box1 = Arc::<RotateY>::new(RotateY::new(box1.clone(), 0.0));
            let box1 =
                Arc::<Translate>::new(Translate::new(box1.clone(), &Vec3::new(0.0, 0.0, 0.0)));
            let box1 = Arc::<ConstantMedium>::new(ConstantMedium::new(box1.clone(), 0.01, v1));
            boxes1.push(box1); */

            boxes1.push(Arc::<Boxes>::new(Boxes::new(
                &Vec3::new(x0, y0, z0),
                &Vec3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    objects.push(Arc::<BvhNode>::new(BvhNode::new_list(boxes1, 0.0, 1.0)));

    let center1 = Vec3::new(440.0, 370.0, 400.0);
    let center2 = center1 + Vec3::new(40.0, 40.0, 0.0);

    // cloud
    let path = Path::new("cloud.jpg");

    let imgtext1 = Arc::<ImageTexture>::new(ImageTexture::new(path));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(220.0, 280.0, 300.0),
        70.0,
        Arc::<Lambertian>::new(Lambertian::new(imgtext1)),
    )));

    // dielectric
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::<Dielectric>::new(Dielectric::new(1.5)),
    )));

    // metal
    objects.push(Arc::<MovingSphere>::new(MovingSphere::new(
        center1,
        center2,
        0.0,
        1.0,
        30.0,
        Arc::<Metal>::new(Metal::new(Vec3::new(0.7, 0.3, 0.1), 10.0)),
    )));

    // fog
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(360.0, 150.0, 145.0),
        70.0,
        Arc::<Dielectric>::new(Dielectric::new(1.5)),
    ));
    objects.push(boundary.clone());
    objects.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.2, 0.4, 0.9))),
    )));

    // fog
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        5000.0,
        Arc::<Dielectric>::new(Dielectric::new(1.5)),
    ));
    objects.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.0001,
        Arc::<Solid>::new(Solid::new(Vec3::new(1.0, 1.0, 1.0))),
    )));

    // earth
    let path = Path::new("input.jpg");

    let imgtext = Arc::<ImageTexture>::new(ImageTexture::new(path));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(410.0, 200.0, 400.0),
        100.0,
        Arc::<Lambertian>::new(Lambertian::new(imgtext)),
    )));

    // perlin
    /* let pn = Perlin::new();
    let pertext = Arc::<Noise>::new(Noise::new(pn, 0.1));
    objects.push(Arc::<Sphere>::new(Sphere::new(
        Vec3::new(220.0, 280.0, 300.0),
        80.0,
        Arc::<Lambertian>::new(Lambertian::new(pertext)),
    ))); */

    // 多球立方体
    let mut boxes2 = Hlist::new(true);
    let vw = Arc::<Solid>::new(Solid::new(Vec3::new(0.73, 0.73, 0.73)));
    let white = Arc::<Lambertian>::new(Lambertian::new(vw));

    let ns: i32 = 1000;
    for _j in 0..ns {
        boxes2.push(Arc::<Sphere>::new(Sphere::new(
            random_limit(0.0, 165.0),
            10.0,
            white.clone(),
        )));
    }

    objects.push(Arc::<Translate>::new(Translate::new(
        Arc::<RotateY>::new(RotateY::new(
            Arc::<BvhNode>::new(BvhNode::new_list(boxes2, 0.0, 1.0)),
            15.0,
        )),
        &Vec3::new(-100.0, 290.0, 395.0),
    )));

    objects
} */

fn cloud() -> Hlist {
    let mut objects = Hlist::new(true);
    let mut group = Hlist::new(true);
    let mut boxes1 = Hlist::new(true);

    // light
    let vl = Arc::<Solid>::new(Solid::new(Vec3::new(7.0, 7.0, 7.0)));
    let light = Arc::<Diffuse>::new(Diffuse::new(vl));
    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        203.0, 343.0, 227.0, 332.0, 554.0, light,
    )));

    // background && ground
    let s1 = Arc::<Solid>::new(Solid::new(Vec3::new(0.498, 0.533, 0.796)));
    let ground = Arc::<Lambertian>::new(Lambertian::new(s1));

    let vb = Arc::<Solid>::new(Solid::new(Vec3::new(0.184, 0.2157, 0.4588)));
    let blue = Arc::<Lambertian>::new(Lambertian::new(vb));

    let boxex_per_side = 20;
    for _i in 0..boxex_per_side {
        for _j in 0..boxex_per_side {
            let w: f64 = 100.0;
            let x0: f64 = -1000.0 + w * _i as f64;
            let z0: f64 = -1000.0 + w * _j as f64;
            let y0: f64 = 0.0;
            let x1: f64 = x0 + w;
            let y1: f64 = random_double_limit(1.0, 101.0);
            let z1: f64 = z0 + w;

            /* let v1 = Arc::<Solid>::new(Solid::new(Vec3::new(1.0, 1.0, 1.0)));

            let box1 = Arc::<Boxes>::new(Boxes::new(
                &Vec3::new(x0, y0, z0),
                &Vec3::new(x1, y1, z1),
                ground.clone(),
            ));
            let box1 = Arc::<RotateY>::new(RotateY::new(box1.clone(), 0.0));
            let box1 =
                Arc::<Translate>::new(Translate::new(box1.clone(), &Vec3::new(0.0, 0.0, 0.0)));
            let box1 = Arc::<ConstantMedium>::new(ConstantMedium::new(box1.clone(), 0.01, v1));
            boxes1.push(box1); */

            boxes1.push(Arc::<Boxes>::new(Boxes::new(
                &Vec3::new(x0, y0, z0),
                &Vec3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }
    objects.push(Arc::<BvhNode>::new(BvhNode::new_list(boxes1, 0.0, 1.0)));

    // background picture moon
    let path = Path::new("moon.jpg");
    let imgtext = Arc::<ImageTexture>::new(ImageTexture::new(path));
    let moon = Arc::<Lambertian>::new(Lambertian::new(imgtext));

    objects.push(Arc::<Yzrect>::new(Yzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        blue.clone(),
    )));

    objects.push(Arc::<Yzrect>::new(Yzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        blue.clone(),
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        blue.clone(),
    )));

    objects.push(Arc::<Xyrect>::new(Xyrect::new(
        //背景
        0.0, 555.0, 0.0, 555.0, 555.0, moon,
    )));

    objects.push(Arc::<Xzrect>::new(Xzrect::new(
        0.0, 555.0, 0.0, 555.0, 555.0, blue,
    )));

    let v = Arc::<Solid>::new(Solid::new(Vec3::new(0.6, 0.6, 0.6)));
    let van = Arc::<Solid>::new(Solid::new(Vec3::new(0.45, 0.45, 0.45)));
    //let _vanan = Arc::<Solid>::new(Solid::new(Vec3::new(0.4, 0.4, 0.4)));

    let path = Path::new("surface.jpg");
    let imgtext = Arc::<ImageTexture>::new(ImageTexture::new(path));
    let surface = Arc::<Diffuse>::new(Diffuse::new(imgtext));

    // 1左下角
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(112.0, 201.0, 155.0),
        40.0,
        Arc::<Diffuse>::new(Diffuse::new(v.clone())),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.98))),
    )));

    // 2右下角
    // cloud
    let path = Path::new("Mercury.jpg");
    let imgtext1 = Arc::<ImageTexture>::new(ImageTexture::new(path));
    let cloud = Arc::<Diffuse>::new(Diffuse::new(imgtext1));

    let boundary = Arc::<Sphere>::new(Sphere::new(Vec3::new(433.0, 171.0, 180.0), 40.0, cloud));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.98))),
    )));

    // 中间 1
    //let iso = Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.886)));
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(145.0, 184.0, 110.0),
        33.0,
        Arc::<Dielectric>::new(Dielectric::new(1.5)),
    ));
    group.push(boundary);
    /* group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.98))),
    ))); */

    // 2
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(184.0, 196.0, 180.0),
        34.0,
        Arc::<Diffuse>::new(Diffuse::new(v.clone())),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.99))),
    )));

    // 3
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(225.0, 180.0, 150.0),
        34.0,
        Arc::<Dielectric>::new(Dielectric::new(3)),
    ));
    group.push(boundary);
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.976))),
    )));

    // 4
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(276.0, 184.0, 145.0),
        34.0,
        Arc::<Diffuse>::new(Diffuse::new(van)),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.966))),
    )));

    // 5
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(327.0, 173.0, 150.0),
        38.0,
        Arc::<Metal>::new(Metal::new(Vec3::new(0.98, 0.98, 0.98), 10.0)),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.98))),
    )));

    // 6
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(384.0, 206.0, 145.0),
        34.0,
        Arc::<Diffuse>::new(Diffuse::new(v.clone())),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.98))),
    )));

    // 7
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(171.0, 241.0, 170.0),
        33.0,
        Arc::<Diffuse>::new(Diffuse::new(v.clone())),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.956))),
    )));

    // 8
    let boundary = Arc::<Sphere>::new(Sphere::new(Vec3::new(229.0, 249.0, 110.0), 55.0, surface));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.926))),
    )));

    // 9
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(304.0, 234.0, 155.0),
        35.0,
        Arc::<Diffuse>::new(Diffuse::new(v.clone())),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.926))),
    )));

    // 10
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(348.0, 256.0, 170.0),
        35.0,
        Arc::<Dielectric>::new(Dielectric::new(3)),
    ));
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.936))),
    )));

    // 11
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(412.0, 267.0, 150.0),
        37.5,
        Arc::<Dielectric>::new(Dielectric::new(3)),
    ));
    group.push(boundary);
    /* group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.956))),
    ))); */

    // 12
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(298.0, 289.0, 170.0),
        37.5,
        Arc::<Dielectric>::new(Dielectric::new(1.5)),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(1.0, 0.98, 0.95))),
    )));

    // 13
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(379.0, 300.0, 140.0),
        34.0,
        Arc::<Diffuse>::new(Diffuse::new(v.clone())),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.986))),
    )));

    // 14
    let boundary = Arc::<Sphere>::new(Sphere::new(
        Vec3::new(326.0, 317.0, 145.0),
        38.0,
        Arc::<Diffuse>::new(Diffuse::new(v)),
    ));
    group.push(boundary.clone());
    group.push(Arc::<ConstantMedium>::new(ConstantMedium::new(
        boundary,
        0.2,
        Arc::<Solid>::new(Solid::new(Vec3::new(0.98, 0.98, 0.956))),
    )));

    // metal
    let center1 = Vec3::new(440.0, 370.0, 400.0);
    let center2 = center1 + Vec3::new(40.0, 40.0, 0.0);
    objects.push(Arc::<MovingSphere>::new(MovingSphere::new(
        center1,
        center2,
        0.0,
        1.0,
        45.0,
        Arc::<Metal>::new(Metal::new(Vec3::new(0.7, 0.3, 0.1), 10.0)),
    )));

    objects.push(Arc::<BvhNode>::new(BvhNode::new_list(group, 0.0, 1.0)));
    objects
}
