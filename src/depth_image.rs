use std::fmt::Debug;

use super::helpers;
use disage::{self, Dimensions, DiscreteImage};
use image::{self, ImageBuffer, Pixel, Rgb};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthPixel<P: Pixel> {
    pub pixel: P,
    pub depth: u32,
}

impl<P: Pixel> DepthPixel<P> {
    pub fn new(pixel: P, depth: u32) -> DepthPixel<P> {
        DepthPixel { pixel, depth }
    }
}

pub struct DepthImage<B> {
    pub depth: Vec<Vec<u32>>,
    pub pixels: B,
}
#[allow(dead_code)]
impl DepthImage<ImageBuffer<Rgb<u16>, Vec<u16>>> {
    pub fn from_rgb16_relative(
        main_img: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        additional_img: &ImageBuffer<Rgb<u16>, Vec<u16>>,
        precision: [u16; 3],
    ) -> Self {
        let (h, w) = (main_img.height(), main_img.width());
        let (ha, wa) = (additional_img.height(), additional_img.width());
        let discrete = disage::open::rgb16(
            main_img.clone(),
            precision.clone(),
            disage::hashers::MedianBrightnessHasher {},
            18,
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
            max / 20,
            precision,
            5
        );
        DepthImage {
            depth: helpers::replace_none_with(
                &helpers::depthp_to_array(&dpix, Dimensions::new(h, w)),
                u32::MIN,
            ),
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
            5
        );
        DepthImage {
            depth: helpers::replace_none_with(
                &helpers::depthp_to_array(&dpix, Dimensions::new(h, w)),
                0,
            ),
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
}
