use disage::{self, Dimensions, DiscretePixel, Position};
use image::{self, ImageBuffer, Luma, Rgb};
use std::time::Instant;

pub mod preparations;

pub mod helpers;

pub mod depth_hasher;

fn main() {
    let img1: ImageBuffer<Rgb<u16>, Vec<u16>> = image::io::Reader::open("inputs/main3.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb16();
    let img2: ImageBuffer<Rgb<u16>, Vec<u16>> = image::io::Reader::open("inputs/sub3.jpg")
        .unwrap()
        .decode()
        .unwrap()
        //.resize(2000, 2000, image::imageops::Gaussian)
        .to_rgb16();
    let img1 = preparations::normalize_brightness_rgb16(&img1, &img2);
    let raw_pixels: Vec<[u16; 3]> = img1.pixels().map(|f| f.0).collect();
    println!("Started creating...");
    let precision = [600,600,600];
    println!("Precision : {:?}", precision);
    let now = Instant::now();
    let hasher = depth_hasher::DepthHasher::from_additional(&img2, Dimensions::new(img1.height(),img1.width()), precision.clone());
    let checker = depth_hasher::DepthChecker{precision : 100u32};
    let discrete = disage::discrete_image::DiscreteImage::new(raw_pixels,hasher,img1.width(),checker,14,20);
    println!("Created, elapsed : {}", now.elapsed().as_secs_f32());
    disage::converters::to_luma8_from32(&helpers::broaden_depth(&discrete.collect(None)))
        .save("outputs/map_weird.jpg")
        .unwrap();
    println!("Hello, world!");
}

#[cfg(test)]
mod tests;
