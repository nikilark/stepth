use disage;
use preparations::*;

pub mod preparations {
    use image::{self, ImageBuffer, Luma, Rgb};
pub fn normalize_brightness_luma16(
    img1: &ImageBuffer<Luma<u16>, Vec<u16>>,
    img2: &ImageBuffer<Luma<u16>, Vec<u16>>,
) -> ImageBuffer<Luma<u16>, Vec<u16>> {
    let fbr: u64 = img1
        .pixels()
        .map(|f| {
            let t: u64 = f.0[0].clone().into();
            t
        })
        .sum();
    let sbr: u64 = img2
        .pixels()
        .map(|f| {
            let t: u64 = f.0[0].clone().into();
            t
        })
        .sum();
    let fbr = fbr / img1.len() as u64;
    let sbr = sbr / img2.len() as u64;
    let diff: f64 = sbr as f64 / fbr as f64;
    let mut res = img1.clone();
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
    img1.pixels().for_each(|f| {
        rgb1[0] += f.0[0] as f64;
        rgb1[1] += f.0[1] as f64;
        rgb1[2] += f.0[2] as f64;
    });
    img2.pixels().for_each(|f| {
        rgb2[0] += f.0[0] as f64;
        rgb2[1] += f.0[1] as f64;
        rgb2[2] += f.0[2] as f64;
    });
    for (array, l) in vec![(&mut rgb1, img1.len()), (&mut rgb2, img2.len())] {
        for i in 0..3 {
            array[i] /= l as f64;
        }
    }
    let diff = [rgb2[0] / rgb1[0], rgb2[1] / rgb1[1], rgb2[2] / rgb1[2]];
    let mut res = img1.clone();
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

fn main() {
    // let hasher = disage::hashers::BrightnessHasher{};
    // let img = disage::open::rgb16("./inputs/1.png",[200,200,200], hasher).expect("Failed to create discrete img");
    // println!("Compression : {}%", img.compression());
    // let outp_img = disage::converters::to_rgb8_from16(&img.collect(None));
    // outp_img.save("./outputs/6.jpg").expect("Failed to save output image");
    let mut img1 = image::io::Reader::open("inputs/1.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb16();
    img1.pixels_mut().for_each(|f| {
        for i in 0..3 {
            f.0[i] = ((f.0[i]) as f64 * 1.5) as u16
        }
    });
    let img2 = image::io::Reader::open("inputs/1.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb16();
    let img3 = normalize_brightness_rgb16(&img1, &img2);
    rgb16_to8(&img1)
        .save("outputs/br.jpg")
        .expect("Failed to save image");
    rgb16_to8(&img2)
        .save("outputs/br2.jpg")
        .expect("Failed to save image");
    rgb16_to8(&img3)
        .save("outputs/br3.jpg")
        .expect("Failed to save image");
    println!("Hello, world!");
}
