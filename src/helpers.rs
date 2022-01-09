use std::cmp::Ordering;
use std::fmt::Debug;
use disage::pixels::PixelOpps;
type DPixelDis<T> = (DiscretePixel<T>, Option<u32>);
use super::*;
use rayon::prelude::*;

pub fn precision_rgb16(img: &ImageBuffer<Rgb<u16>, Vec<u16>>, percent: f32) -> [u16; 3] {
    if percent < 0.0 || percent > 1.0 {
        println!("Wrong percent value : {}, should be in [0.0, 1.0]", percent);
        return [0, 0, 0];
    }
    let mut previous = img.get_pixel(0, 0).clone().0;
    let mut diffs: Vec<[u16; 3]> = img
        .pixels()
        .map(|f| {
            let d = f.0.substract(previous.clone());
            previous = f.0.clone();
            d
        })
        .collect();
    diffs.sort_by(|a, b| {
        if a == b {
            return Ordering::Equal;
        }
        if a.lt(b) {
            return Ordering::Less;
        }
        return a.cmp(b);
    });
    diffs[(diffs.len() as f32 * percent) as usize].clone()
}

pub fn precision_luma16(img: &ImageBuffer<Luma<u16>, Vec<u16>>, percent: f32) -> u16 {
    if percent < 0.0 || percent > 1.0 {
        println!("Wrong percent value : {}, should be in [0.0, 1.0]", percent);
        return 0;
    }
    let mut pixels: Vec<u16> = img.pixels().map(|f| f.0[0]).collect();
    pixels.sort();
    pixels[(pixels.len() as f32 * percent) as usize].clone()
}

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

pub fn distance_dot_array<T: Clone + PixelOpps<T> + Debug>(
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
                            let dist = distance_dot_dot(from, Position::new(j as u32, i as u32));
                            //println!("Found : this = {:?}, eq = {:?}, dist = {}", v, what, dist);
                            return Some(dist);
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
                            let dist = distance_dot_dot(from, Position::new(j as u32, i as u32));
                            //println!("Found : this = {:?}, eq = {:?}, dist = {}", v, what, dist);
                            return Some(dist);
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

pub fn smooth_depth(depth: &Vec<Option<u32>>, kernel: usize) -> Vec<Option<u32>> {
    let mut res = depth.clone();
    let len = depth.len();
    let chunk_size = 1 + len / 8;
    res.par_chunks_mut(chunk_size)
        .enumerate()
        .for_each(|(chunk_index, chunk)| {
            chunk.iter_mut().enumerate().for_each(|(index, v)| {
                let final_index = chunk_index * chunk_size + index;
                let window: &[Option<u32>] = match (
                    final_index > kernel / 2,
                    len as i64 - final_index as i64 > (kernel / 2) as i64,
                ) {
                    (true, true) => &depth[final_index - kernel / 2..final_index + kernel / 2],
                    (false, true) => &depth[0..chunk_size.max(len)],
                    (true, false) => {
                        &depth[(0i64.max(final_index as i64 - kernel as i64)) as usize..len]
                    }
                    (false, false) => &depth[0..len],
                };
                let somes: Vec<u64> = window
                    .iter()
                    .filter_map(|v| v.and_then(|f| Some(f as u64)))
                    .collect();
                if somes.len() != 0 {
                    *v = Some((somes.iter().sum::<u64>() / somes.len() as u64) as u32);
                }
            })
        });
    res
}

pub fn distance_discrete_pixels<
    T: Clone + Copy + std::marker::Sync + PixelOpps<T> + std::marker::Send + Debug,
>(
    pixels: &Vec<DiscretePixel<T>>,
    img_size: Dimensions,
    array: &Vec<Vec<T>>,
    max: u32,
    precision: T,
    smoothing : usize,
) -> Vec<DPixelDis<T>> {
    let arr_dim = Dimensions {
        height: array.len() as u32,
        width: array[0].len() as u32,
    };
    let mut distances: Vec<Option<u32>> = vec![None; pixels.len()];
    let chunk_size = 1 + pixels.len() / 8;
    let last_chunk = distances.len() / chunk_size;
    let now = Instant::now();
    distances
        .par_chunks_mut(chunk_size)
        .enumerate()
        .for_each(|(ci, c)| {
            if ci == last_chunk {
                let bar = indicatif::ProgressBar::new(c.len() as u64);
                c.iter_mut().enumerate().for_each(|(index, val)| {
                    let pix = &pixels[chunk_size * ci + index];
                    let rel_pos = relative_pos(pix.position, img_size, arr_dim);
                    *val = distance_dot_array(&pix.value, array, rel_pos, max, precision);
                    bar.inc(1);
                })
            } else {
                c.iter_mut().enumerate().for_each(|(index, val)| {
                    let pix = &pixels[chunk_size * ci + index];
                    let rel_pos = relative_pos(pix.position, img_size, arr_dim);
                    *val = distance_dot_array(&pix.value, array, rel_pos, max, precision);
                })
            }
        });
    println!("Found distances, elapsed : {}", now.elapsed().as_secs_f32());
    let now = Instant::now();
    let smoothed = smooth_depth(&distances, smoothing);
    println!("Smoothed, elapsed : {}", now.elapsed().as_secs_f32());
    pixels
        .to_vec()
        .into_iter()
        .zip(smoothed.into_iter())
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