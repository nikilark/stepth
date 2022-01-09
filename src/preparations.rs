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
    if (1f64 - diff).abs() < 0.34 {
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
    if (1f64 - diff[0]).abs() < 0.34
        && (1f64 - diff[1]).abs() < 0.34
        && (1f64 - diff[2]).abs() < 0.34
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
