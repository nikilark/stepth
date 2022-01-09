use disage::{self, Dimensions, DiscretePixel, Position};
use image::{self, ImageBuffer, Luma, Rgb};
use std::time::Instant;

pub mod preparations;

pub mod helpers;

pub mod depth_image;

fn main() {
    let img1: ImageBuffer<Rgb<u16>, Vec<u16>> = image::io::Reader::open("inputs/main2.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb16();
    let img2: ImageBuffer<Rgb<u16>, Vec<u16>> = image::io::Reader::open("inputs/sub2.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .resize(1000, 1000, image::imageops::Gaussian)
        .to_rgb16();
    let img1 = preparations::normalize_brightness_rgb16(&img1, &img2);
    println!("Started creating...");
    let precision = helpers::precision_rgb16(&img1, 0.8);
    println!("Precision : {:?}", precision);
    let now = Instant::now();
    let di = depth_image::DepthImage::from_rgb16_relative(&img1, &img2, precision.clone());
    println!("Created, elapsed : {}", now.elapsed().as_secs_f32());
    di.broaden_depth()
        .depth_image()
        .save("outputs/map.jpg")
        .unwrap();
    println!("Hello, world!");
}

#[cfg(test)]
mod tests;
