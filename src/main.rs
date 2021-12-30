use std::fmt::Debug;

use disage::{self, Dimensions, DiscreteImage, DiscretePixel, Position};
use image::{self, ImageBuffer, Luma, Pixel, Rgb};
use preparations::*;
use rayon::prelude::*;

type DPixelDis<T> = (DiscretePixel<T>, Option<u32>);

pub mod preparations {
    use image::{self, ImageBuffer, Luma, Rgb};
    pub fn normalize_brightness_luma16(
        img1: &ImageBuffer<Luma<u16>, Vec<u16>>,
        img2: &ImageBuffer<Luma<u16>, Vec<u16>>,
    ) -> ImageBuffer<Luma<u16>, Vec<u16>> {
        let (fbr, sbr): (u64, u64) = rayon::join(
            || {
                img1.pixels()
                    .map(|f| {
                        let t: u64 = f.0[0].clone().into();
                        t
                    })
                    .sum()
            },
            || {
                img2.pixels()
                    .map(|f| {
                        let t: u64 = f.0[0].clone().into();
                        t
                    })
                    .sum()
            },
        );
        let fbr = fbr / img1.len() as u64;
        let sbr = sbr / img2.len() as u64;
        let diff: f64 = sbr as f64 / fbr as f64;
        let mut res = img1.clone();
        if (1f64 - diff).abs() < 0.1 {
            println!("No need to change brightness");
            return res;
        }
        res.pixels_mut()
            .for_each(|f| f.0[0] = ((f.0[0] as f64) * diff) as u16);
        res
    }

    pub fn luma16_to8(img: &ImageBuffer<Luma<u16>, Vec<u16>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let mut res: image::GrayImage = image::ImageBuffer::new(img.width(), img.height());
        res.enumerate_pixels_mut()
            .for_each(|(x, y, pix)| pix.0[0] = (img.get_pixel(x, y).0[0] >> 8) as u8);
        res
    }

    pub fn normalize_brightness_rgb16(
        img1: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        img2: &ImageBuffer<Rgb<u16>, Vec<u16>>,
    ) -> ImageBuffer<Rgb<u16>, Vec<u16>> {
        let mut rgb1 = [0f64, 0.0, 0.0];
        let mut rgb2 = rgb1.clone();
        rayon::join(
            || {
                img1.pixels().for_each(|f| {
                    rgb1[0] += f.0[0] as f64;
                    rgb1[1] += f.0[1] as f64;
                    rgb1[2] += f.0[2] as f64;
                })
            },
            || {
                img2.pixels().for_each(|f| {
                    rgb2[0] += f.0[0] as f64;
                    rgb2[1] += f.0[1] as f64;
                    rgb2[2] += f.0[2] as f64;
                })
            },
        );
        for (array, l) in vec![(&mut rgb1, img1.len()), (&mut rgb2, img2.len())] {
            for i in 0..3 {
                array[i] /= l as f64;
            }
        }
        let diff = [rgb2[0] / rgb1[0], rgb2[1] / rgb1[1], rgb2[2] / rgb1[2]];
        let mut res = img1.clone();
        if (1f64 - diff[0]).abs() < 0.1
            && (1f64 - diff[1]).abs() < 0.1
            && (1f64 - diff[2]).abs() < 0.1
        {
            println!("No need to change brightness");
            return res;
        }
        res.pixels_mut().for_each(|f| {
            for i in 0..3 {
                f.0[i] = ((f.0[i] as f64) * diff[i]) as u16;
            }
        });
        res
    }

    pub fn rgb16_to8(img: &ImageBuffer<Rgb<u16>, Vec<u16>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut res: image::RgbImage = image::ImageBuffer::new(img.width(), img.height());
        res.enumerate_pixels_mut().for_each(|(x, y, pix)| {
            let p = img.get_pixel(x, y).0;
            pix.0[0] = (p[0] >> 8) as u8;
            pix.0[1] = (p[1] >> 8) as u8;
            pix.0[2] = (p[2] >> 8) as u8;
        });
        res
    }
}

pub mod helpers {
    use super::*;

    pub fn relative_pos(pos: Position, size: Dimensions, size_to: Dimensions) -> Position {
        let (rx, ry) = (
            (pos.x as f64 / size.width as f64),
            (pos.y as f64 / size.height as f64),
        );
        Position {
            x: (size_to.width as f64 * rx) as u32,
            y: (size_to.height as f64 * ry) as u32,
        }
    }

    pub fn distance_dot_dot(f: Position, s: Position) -> u32 {
        let (x1, y1) = (f.x as i64, f.y as i64);
        let (x2, y2) = (s.x as i64, s.y as i64);
        (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f64).sqrt() as u32
    }

    pub fn distance_dot_array<T: Clone + disage::PixelOpps<T> + Debug>(
        what: &T,
        array: &Vec<Vec<T>>,
        from: Position,
        max: u32,
        presition: T,
    ) -> Option<u32> {
        let (x, y): (u32, u32) = from.tuplexy();
        let x = x as i64;
        let y = y as i64;
        let get2d = |array: &Vec<Vec<T>>, i: i64, j: i64| match array.get(i as usize) {
            Some(t) => match t.get(j as usize) {
                Some(v) => Some(v.clone()),
                None => None,
            },
            None => None,
        };
        for i in 0..max as i64 {
            let mut found = false;
            for i in [y - i, y + i] {
                for j in x - i..x + i + 1 {
                    match get2d(&array, i, j) {
                        Some(v) => {
                            found = true;

                            if v.clone().substract(what.clone()).lt(presition.clone()) {
                                return Some(distance_dot_dot(
                                    from,
                                    Position::new(j as u32, i as u32),
                                ));
                            }
                        }
                        None => continue,
                    }
                }
            }
            for j in [x - i, x + i] {
                for i in y - i..y + i + 1 {
                    match get2d(&array, i, j) {
                        Some(v) => {
                            found = true;
                            if v.clone().substract(what.clone()).lt(presition.clone()) {
                                return Some(distance_dot_dot(
                                    from,
                                    Position::new(j as u32, i as u32),
                                ));
                            }
                        }
                        None => continue,
                    }
                }
            }
            if !found {
                break;
            }
        }
        None
    }

    pub fn distance_discrete_pixels<
        T: Clone + Copy + std::marker::Sync + disage::PixelOpps<T> + std::marker::Send + Debug,
    >(
        pixels: &Vec<DiscretePixel<T>>,
        img_size: Dimensions,
        array: &Vec<Vec<T>>,
        max: u32,
        precision: T,
    ) -> Vec<DPixelDis<T>> {
        let arr_dim = Dimensions {
            height: array.len() as u32,
            width: array[0].len() as u32,
        };
        let mut distances: Vec<Option<u32>> = vec![None; pixels.len()];
        let chunk_size = 1 + pixels.len() / 20;
        distances
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(ci, c)| {
                c.iter_mut().enumerate().for_each(|(index, val)| {
                    let pix = &pixels[chunk_size * ci + index];
                    let rel_pos = relative_pos(pix.position, img_size, arr_dim);
                    *val = distance_dot_array(&pix.value, array, rel_pos, max, precision);
                })
            });
        pixels
            .to_vec()
            .into_iter()
            .zip(distances.into_iter())
            .collect()
    }

    pub fn depthp_to_array<T: Clone>(
        pixels: &Vec<DPixelDis<T>>,
        size: Dimensions,
    ) -> Vec<Vec<Option<u32>>> {
        let mut res = vec![vec![Option::<u32>::None; size.width as usize]; size.height as usize];
        for (pix, dis) in pixels {
            let (x, y): (u32, u32) = pix.position.tuplexy();
            let (h, w): (u32, u32) = pix.size.tuplehw();
            for i in y..y + h {
                for j in x..x + w {
                    res[i as usize][j as usize] = dis.clone();
                }
            }
        }
        res
    }

    pub fn replace_none_with(array: &Vec<Vec<Option<u32>>>, default: u32) -> Vec<Vec<u32>> {
        array
            .iter()
            .map(|op| {
                op.iter()
                    .map(|v| match v {
                        Some(p) => p.clone(),
                        None => default,
                    })
                    .collect()
            })
            .collect()
    }

    pub fn neighbours<T: Clone>(pos: Position, array: &Vec<Vec<T>>, close: usize) -> Vec<T> {
        if close == 0 {
            return match array
                .get(pos.y as usize)
                .and_then(|f| f.get(pos.x as usize))
            {
                Some(v) => vec![v.clone()],
                None => vec![],
            };
        }
        let mut res = Vec::with_capacity(8 * close);
        let (x, y) = (pos.x as i64, pos.y as i64);
        for i in [y - close as i64, y + close as i64] {
            if i < 0 {
                continue;
            }
            for j in x - close as i64..x + 1 + close as i64 {
                if j < 0 {
                    continue;
                }
                res.push(array.get(i as usize).and_then(|f| f.get(j as usize)));
            }
        }
        for j in [x - close as i64, x + close as i64] {
            if j < 0 {
                continue;
            }
            for i in y - close as i64 + 1..y + close as i64 {
                if i < 0 {
                    continue;
                }
                res.push(array.get(i as usize).and_then(|f| f.get(j as usize)));
            }
        }

        res.iter()
            .filter_map(|f| f.and_then(|f| Some(f.clone())))
            .collect()
    }

    pub fn fix_none(array: &mut Vec<Vec<Option<u32>>>) {
        let mut found = false;
        for r in array.iter() {
            for el in r {
                if el.is_some() {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return;
        }
        let (h, w) = (array.len(), array[0].len());
        let min_dim = if h < w { h } else { w };
        let arr_clone = array.clone();
        let chunk_size = 1 + h / 20;
        array
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(ci, c)| {
                c.iter_mut().enumerate().for_each(|(y, v)| {
                    let y = chunk_size * ci + y;
                    v.iter_mut().enumerate().for_each(|(x, f)| {
                        if f.is_some() {
                            return;
                        }
                        for i in 1..min_dim {
                            let mut neighbours: Vec<u32> = helpers::neighbours(
                                Position::new(x as u32, y as u32),
                                &arr_clone,
                                i,
                            )
                            .into_iter()
                            .filter_map(|f| f)
                            .collect();
                            if neighbours.len() == 0 {
                                continue;
                            }
                            neighbours.sort();
                            let l = neighbours.len();
                            *f = neighbours.get((l - 1) / 2).and_then(|f| Some(f.clone()));
                            return;
                        }
                    })
                })
            });
    }

    pub fn broaden_depth(depth: &Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        let min = depth
            .iter()
            .map(|f| f.iter().min().unwrap_or(&0))
            .min()
            .unwrap_or(&0)
            .clone();
        let max = depth
            .iter()
            .map(|f| f.iter().max().unwrap_or(&u32::MAX))
            .max()
            .unwrap_or(&u32::MAX)
            .clone();
        let delta = (max - min) as f64;
        let chunk_size = depth.len() / 20;
        let mut res = depth.clone();
        res.par_chunks_mut(1 + chunk_size).for_each(|c| {
            c.iter_mut().for_each(|op| {
                op.iter_mut().for_each(|v| {
                    *v = (((v.clone() - min) as f64 / delta) * u32::MAX as f64) as u32
                })
            })
        });
        res
    }

    pub fn smooth_depth(depth: &Vec<Vec<u32>>, kernel: usize) -> Vec<Vec<u32>> {
        let mut res = depth.clone();
        let chunk_size = 1 + depth.len() / 20;
        res.par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(ci, c)| {
                c.iter_mut().enumerate().for_each(|(y, r)| {
                    let y = y + ci * chunk_size;
                    r.iter_mut().enumerate().for_each(|(x, v)| {
                        let mut neighbrs = Vec::new();
                        for k in 0..kernel + 1 {
                            neighbrs.append(&mut neighbours(
                                Position::new(x as u32, y as u32),
                                &depth,
                                k,
                            ));
                        }
                        neighbrs.sort();
                        *v = neighbrs[(neighbrs.len() - 1) / 2];
                    })
                })
            });
        res
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DepthPixel<P: Pixel> {
    pub pixel: P,
    pub depth: u32,
}

impl<P: Pixel> DepthPixel<P> {
    pub fn new(pixel: P, depth: u32) -> DepthPixel<P> {
        DepthPixel { pixel, depth }
    }
}

struct DepthImage<B> {
    pub depth: Vec<Vec<u32>>,
    pub pixels: B,
}
#[allow(dead_code)]
impl DepthImage<ImageBuffer<Rgb<u16>, Vec<u16>>> {
    pub fn from_rgb16_relative(
        main_img: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        additional_img: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        precision_dis: [u16; 3],
        precision: [u16; 3],
    ) -> Self {
        let adjusted_img = normalize_brightness_rgb16(&main_img, &additional_img);
        println!("Brightness");
        let (h, w) = (adjusted_img.height(), adjusted_img.width());
        let (ha, wa) = (additional_img.height(), additional_img.height());
        println!("Main : ({}, {}), Sub : ({}, {})", h, w, ha, wa);
        let discrete = disage::open::rgb16(
            adjusted_img,
            precision_dis,
            disage::hashers::BrightnessHasher {},
        );
        disage::converters::to_rgb8_from16(&discrete.clone().collect(Some([0u16, 0, 0])))
            .save("outputs/wtf.jpg")
            .unwrap();
        println!(
            "Discrete, count : {}, compression : {}",
            discrete.group_count(),
            discrete.compression()
        );
        let max = match ha > wa {
            true => ha,
            _ => wa,
        };
        let pixels: Vec<Rgb<u16>> = additional_img.pixels().map(|f| f.clone()).collect();
        let dpix = helpers::distance_discrete_pixels(
            &discrete.pixels(),
            Dimensions::new(h, w),
            &disage::DiscreteImage::<u8>::pixels_to_array(&pixels, wa),
            max,
            precision,
        );
        println!("Depth");
        let mut option_depth = helpers::depthp_to_array(&dpix, Dimensions::new(h, w));
        println!("To array");
        helpers::fix_none(&mut option_depth);
        println!("Fix none");
        DepthImage {
            depth: helpers::replace_none_with(&option_depth, 0),
            pixels: main_img.clone(),
        }
    }

    pub fn from_discrete_rgb16_relative(
        discrete: &DiscreteImage<[u16; 3]>,
        additional_img: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        precision: [u16; 3],
    ) -> Self {
        let (h, w) = discrete.size.tuplehw();
        let max = match h > w {
            true => h,
            _ => w,
        };
        let pixels: Vec<Rgb<u16>> = additional_img.pixels().map(|f| f.clone()).collect();
        let dpix = helpers::distance_discrete_pixels(
            &discrete.pixels(),
            Dimensions::new(h, w),
            &disage::DiscreteImage::<u16>::pixels_to_array(&pixels, w),
            max,
            precision,
        );
        let mut option_depth = helpers::depthp_to_array(&dpix, Dimensions::new(h, w));
        helpers::fix_none(&mut option_depth);
        DepthImage {
            depth: helpers::replace_none_with(&option_depth, 0),
            pixels: disage::converters::to_rgb16(&discrete.clone().collect(None)),
        }
    }

    pub fn depth_image(&self) -> image::GrayImage {
        disage::converters::to_luma8_from32(&self.depth)
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> DepthPixel<Rgb<u16>> {
        DepthPixel::new(
            self.pixels.get_pixel(x, y).clone(),
            self.depth[y as usize][x as usize],
        )
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, pix: Rgb<u16>) {
        self.pixels.put_pixel(x, y, pix)
    }

    pub fn put_where(
        &mut self,
        other: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        pred: &dyn Fn(&DepthPixel<Rgb<u16>>) -> bool,
    ) {
        self.pixels.enumerate_pixels_mut().for_each(|(x, y, val)| {
            let d = DepthPixel::new(val.clone(), self.depth[y as usize][x as usize]);
            if pred(&d) {
                *val = other.get_pixel(x, y).clone();
            }
        })
    }

    pub fn broaden_depth(&self) -> Self {
        println!("Broading");
        DepthImage {
            depth: helpers::broaden_depth(&self.depth),
            pixels: self.pixels.clone(),
        }
    }

    pub fn smooth_depth(&self, kernel: usize) -> Self {
        println!("Smoothing with kernel {}", kernel);
        DepthImage {
            depth: helpers::smooth_depth(&self.depth, kernel),
            pixels: self.pixels.clone(),
        }
    }
}

fn main() {
    let img1: ImageBuffer<Rgb<u16>, Vec<u16>> = image::io::Reader::open("inputs/main.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .resize(600, 400, image::imageops::Gaussian)
        .to_rgb16();
    let img2: ImageBuffer<Rgb<u16>, Vec<u16>> = image::io::Reader::open("inputs/sub.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .resize(600, 400, image::imageops::Gaussian)
        .to_rgb16();
    println!(
        "Main : ({}, {}), Sub : ({}, {})",
        img1.height(),
        img1.width(),
        img2.height(),
        img2.width()
    );
    println!("Started creating...");

    let di = DepthImage::from_rgb16_relative(&img1, &img2, [600u16, 600, 600], [500u16, 500, 500]);
    println!("Created");
    di.smooth_depth((img1.height() / 20) as usize)
        .broaden_depth()
        .depth_image()
        .save("outputs/map.jpg")
        .unwrap();
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_none_test() {
        let mut arr = vec![
            vec![None, Some(1u32), Some(2u32)],
            vec![None, None, Some(5)],
            vec![Some(6), Some(7), None],
        ];
        let arr_fixed = vec![
            vec![Some(1), Some(1u32), Some(2u32)],
            vec![Some(6), Some(5), Some(5)],
            vec![Some(6), Some(7), Some(5)],
        ];
        let mut arr_none: Vec<Vec<Option<u32>>> = vec![
            vec![None, None, None],
            vec![None, None, None],
            vec![None, None, None],
        ];
        let arr_none_fixed = arr_none.clone();
        let mut arr_one: Vec<Vec<Option<u32>>> = vec![
            vec![Some(1u32), None, None],
            vec![None, None, None],
            vec![None, None, None],
        ];
        let arr_one_fixed: Vec<Vec<Option<u32>>> = vec![
            vec![Some(1u32), Some(1u32), Some(1u32)],
            vec![Some(1u32), Some(1u32), Some(1u32)],
            vec![Some(1u32), Some(1u32), Some(1u32)],
        ];
        helpers::fix_none(&mut arr);
        helpers::fix_none(&mut arr_none);
        helpers::fix_none(&mut arr_one);
        assert_eq!(arr, arr_fixed);
        assert_eq!(arr_none, arr_none_fixed);
        assert_eq!(arr_one, arr_one_fixed);
    }

    #[test]
    fn neighbours_test() {
        let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
        assert_eq!(helpers::neighbours(Position::new(0, 0), &arr, 0), vec![0]);
        let mut a = helpers::neighbours(Position::new(0, 0), &arr, 1);
        a.sort();
        assert_eq!(a, vec![1, 3, 4]);
        let mut a = helpers::neighbours(Position::new(1, 1), &arr, 1);
        a.sort();
        assert_eq!(a, vec![0, 1, 2, 3, 5, 6, 7, 8]);
        let mut a = helpers::neighbours(Position::new(1, 1), &arr, 10);
        a.sort();
        assert_eq!(a, vec![]);
    }

    #[test]
    fn relative_pos_test() {
        assert_eq!(
            helpers::relative_pos(
                Position::new(5, 5),
                Dimensions::new(10, 10),
                Dimensions::new(20, 20)
            ),
            Position::new(10, 10)
        );
    }

    #[test]
    fn distance_dot_dot_test() {
        assert_eq!(
            helpers::distance_dot_dot(Position::new(0, 0), Position::new(1, 0)),
            1
        );
        assert_eq!(
            helpers::distance_dot_dot(Position::new(0, 0), Position::new(1, 1)),
            1
        );
        assert_eq!(
            helpers::distance_dot_dot(Position::new(0, 0), Position::new(2, 2)),
            2
        );
        assert_eq!(
            helpers::distance_dot_dot(Position::new(0, 0), Position::new(2, 3)),
            3
        );
    }

    #[test]
    fn distance_dot_array_test() {
        let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
        assert_eq!(
            helpers::distance_dot_array(&0, &arr, Position::new(0, 0), 10, 1),
            Some(0)
        );
        assert_eq!(
            helpers::distance_dot_array(&0, &arr, Position::new(1, 1), 10, 1),
            Some(1)
        );
        assert_eq!(
            helpers::distance_dot_array(&0, &arr, Position::new(2, 2), 10, 1),
            Some(2)
        );
        assert_eq!(
            helpers::distance_dot_array(&0, &arr, Position::new(2, 2), 1, 1),
            None
        );
        assert_eq!(
            helpers::distance_dot_array(&0, &arr, Position::new(22, 22), 100, 1),
            None
        );
    }

    #[test]
    fn distance_discrete_pixels_test() {
        let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
        let pix1 = DiscretePixel::new(0u16, Position::new(0, 0), Dimensions::new(100, 100));
        let pix2 = DiscretePixel::new(0u16, Position::new(10, 0), Dimensions::new(100, 100));
        let pix3 = DiscretePixel::new(5u16, Position::new(20, 20), Dimensions::new(100, 100));
        let pix4 = DiscretePixel::new(2u16, Position::new(100, 100), Dimensions::new(100, 100));
        let vpix = vec![pix1.clone(), pix2.clone(), pix3.clone(), pix4.clone()];
        let pos_res = vec![
            (pix1, Some(0u32)),
            (pix2, Some(1)),
            (pix3, Some(1)),
            (pix4, None),
        ];
        assert_eq!(
            helpers::distance_discrete_pixels(&vpix, Dimensions::new(30, 30), &arr, 5, 1),
            pos_res
        );
    }

    #[test]
    fn dpixels_to_array_test() {
        let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
        let pix1 = DiscretePixel::new(0u16, Position::new(0, 0), Dimensions::new(4, 2));
        let pix2 = DiscretePixel::new(0u16, Position::new(2, 0), Dimensions::new(2, 2));
        let pix3 = DiscretePixel::new(10u16, Position::new(2, 2), Dimensions::new(2, 2));
        let v = vec![
            (pix1.clone(), Some(0u32)),
            (pix2.clone(), Some(2)),
            (pix3.clone(), None),
        ];
        let pos_res = vec![
            vec![Some(0u32), Some(0), Some(2), Some(2)],
            vec![Some(0), Some(0), Some(2), Some(2)],
            vec![Some(0), Some(0), None, None],
            vec![Some(0), Some(0), None, None],
        ];
        assert_eq!(
            helpers::distance_discrete_pixels(
                &vec![pix1, pix2, pix3],
                Dimensions::new(3, 3),
                &arr,
                3,
                1
            ),
            v.clone()
        );
        assert_eq!(helpers::depthp_to_array(&v, Dimensions::new(4, 4)), pos_res);
    }
}
