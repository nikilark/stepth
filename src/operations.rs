use image::{self, ImageBuffer, Luma, Rgb};
use rayon::prelude::*;

pub fn normalize_brightness_luma16(
    img1: &ImageBuffer<Luma<u16>, Vec<u16>>,
    img2: &ImageBuffer<Luma<u16>, Vec<u16>>,
    percent: f64,
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
    if (1f64 - diff).abs() < percent {
        return res;
    }
    res.pixels_mut()
        .for_each(|f| f.0[0] = ((f.0[0] as f64) * diff) as u16);
    res
}

pub fn normalize_brightness_rgb16(
    img1: &ImageBuffer<Rgb<u16>, Vec<u16>>,
    img2: &ImageBuffer<Rgb<u16>, Vec<u16>>,
    percent: f64,
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
    if (1f64 - diff[0]).abs() < percent
        && (1f64 - diff[1]).abs() < percent
        && (1f64 - diff[2]).abs() < percent
    {
        return res;
    }
    res.pixels_mut().for_each(|f| {
        for i in 0..3 {
            f.0[i] = ((f.0[i] as f64) * diff[i]) as u16;
        }
    });
    res
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
    let chunk_size = depth.len() / 8;
    let mut res = depth.clone();
    res.par_chunks_mut(1 + chunk_size).for_each(|c| {
        c.iter_mut().for_each(|op| {
            op.iter_mut()
                .for_each(|v| *v = (((v.clone() - min) as f64 / delta) * u32::MAX as f64) as u32)
        })
    });
    res
}

pub fn invert_depth(depth: &Vec<Vec<u32>>) -> Vec<Vec<u32>> {
    let chunk_size = depth.len() / 8;
    let mut res = depth.clone();
    res.par_chunks_mut(1 + chunk_size).for_each(|c| {
        c.iter_mut()
            .for_each(|op| op.iter_mut().for_each(|v| *v = u32::MAX - *v))
    });
    res
}
