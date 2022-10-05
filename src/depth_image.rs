use crate::{
    helpers::{self, distance_dot_dot},
    mask_image::*,
};
use image::{DynamicImage, ImageBuffer, Luma};
use std::collections::HashMap;

#[derive(Clone)]
pub struct DepthImage {
    pub image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub depth: ImageBuffer<image::Luma<u8>, Vec<u8>>,
}

impl DepthImage {
    pub fn open(image_path: &str) -> Self {
        let image = image::open(image_path).unwrap().to_rgba8();
        let depth = ImageBuffer::from_pixel(image.width(), image.height(), Luma([0u8]));
        DepthImage { image, depth }
    }

    pub fn load_depth(
        &mut self,
        depth: ImageBuffer<image::Luma<u8>, Vec<u8>>,
    ) -> Result<(), std::io::Error> {
        if depth.width() == self.width() && depth.height() == self.height() {
            self.depth = depth;
            return Ok(());
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Sizes don't match",
        ))
    }

    pub fn load_depth_from_file(&mut self, depth_path: &str) -> Result<(), std::io::Error> {
        let depth_image = image::open(depth_path);
        if depth_image.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to open image",
            ));
        }
        self.load_depth(depth_image.unwrap().to_luma8())
    }

    pub fn load_depth_from_additional(&mut self, add_path: &str) -> Result<(), std::io::Error> {
        let add_image = image::open(add_path);
        if add_image.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to open image",
            ));
        }
        let add_image = add_image.unwrap().to_rgb8();
        if add_image.width() != self.width() || add_image.height() != self.height() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Sizes don't match",
            ));
        }
        let precision = [5u8; 3];
        let mut discr_main = disage::open::rgb_discrete(
            &image::DynamicImage::ImageRgba8(self.image.clone()).to_rgb8(),
            disage::hashers::MeanBrightnessHasher {},
            disage::checkers::BrightnessChecker { precision },
            (10, 16),
        );
        let discr_add = disage::open::rgb_discrete(
            &add_image,
            disage::hashers::MeanBrightnessHasher {},
            disage::checkers::BrightnessChecker { precision },
            (10, 16),
        );
        let add_array = discr_add.clone().collect();
        disage::converters::to_rgb8(&add_array)
            .save("test123.jpg")
            .unwrap();
        disage::converters::to_rgb8(&discr_main.clone().collect())
            .save("test123_main.jpg")
            .unwrap();
        let mut max: u32 = 0;
        let distances: Vec<u32> = discr_main
            .pixels()
            .iter()
            .map(|p| {
                match helpers::distance_dot_array(
                    &p.value,
                    &add_array,
                    p.position,
                    add_image.width() / 3,
                    precision,
                ) {
                    Some((distance, pos)) => {
                        let res = distance
                            + distance_dot_dot(
                                p.position,
                                discr_add.pixel_at(pos).unwrap().position,
                            );
                        if res > max {
                            max = res
                        }
                        res
                    }
                    _ => u32::MIN,
                }
            })
            .collect();
        discr_main
            .pixels()
            .iter()
            .zip(distances.iter())
            .for_each(|(p, v)| {
                discr_main.modify_leaf(
                    [(*v as u64 * u8::MAX as u64 / max as u64) as u8; 3],
                    p.position,
                );
            });
        disage::converters::to_rgb8(&discr_main.clone().collect())
            .save("test_rgb.jpg")
            .unwrap();
        self.load_depth(disage::converters::to_luma8_from_rgb8(
            &discr_main.collect(),
        ))
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn resize(&mut self, to: disage::Dimensions) {
        self.image = DynamicImage::ImageRgba8(self.image.clone())
            .resize(to.width, to.height, image::imageops::Gaussian)
            .to_rgba8();
        self.depth = DynamicImage::ImageLuma8(self.depth.clone())
            .resize(to.width, to.height, image::imageops::Gaussian)
            .to_luma8();
    }

    pub fn dimensions(&self) -> disage::Dimensions {
        disage::Dimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    pub fn depth_split(&self, zones: u8) -> Vec<(Option<u8>, Option<u8>)> {
        if zones < 2 {
            return vec![(None, None)];
        }
        fn inner(array: &[u8], prev_centers: Vec<u8>) -> Vec<(Option<u8>, Option<u8>)> {
            let mut clusters = HashMap::new();
            prev_centers.iter().for_each(|i| {
                clusters.insert(i.clone(), Vec::new());
            });
            array.iter().for_each(|item| {
                let closest_centroid = prev_centers
                    .iter()
                    .min_by(|x, y| {
                        ((**x as i32) - (*item as i32))
                            .abs()
                            .cmp(&((**y as i32) - (*item as i32)).abs())
                    })
                    .unwrap();
                clusters
                    .get_mut(closest_centroid)
                    .unwrap()
                    .push(item.clone());
            });
            let mut new_centroids = clusters
                .iter()
                .map(|(_, v)| (v.iter().map(|v| *v as usize).sum::<usize>() / v.len()) as u8)
                .collect::<Vec<u8>>();
            new_centroids.sort();
            let mut centroids_didnt_change = true;
            for (new, prev) in prev_centers.iter().zip(new_centroids.iter()) {
                if new != prev {
                    centroids_didnt_change = false;
                    break;
                }
            }
            if centroids_didnt_change {
                return new_centroids
                    .iter()
                    .map(|c| {
                        let v = clusters.get(c).unwrap();
                        (
                            Some(*v.iter().min().unwrap()),
                            Some(*v.iter().max().unwrap()),
                        )
                    })
                    .collect();
            } else {
                return inner(array, new_centroids);
            }
        }
        inner(
            self.depth.as_raw(),
            (u8::MIN..u8::MAX)
                .step_by((255 / (zones - 1)) as usize - 1)
                .collect(),
        )
    }

    pub fn select_foreground(&mut self) -> MaskImage {
        let (from, to) = self.depth_split(2)[0];
        self.slice(from, to)
    }

    pub fn invert_depth(&mut self) {
        self.depth.pixels_mut().for_each(|p| p.0[0] = 255 - p.0[0]);
    }

    pub fn slice(&mut self, from: Option<u8>, to: Option<u8>) -> MaskImage {
        let from_parsed = from.unwrap_or(0);
        let to_parsed = to.unwrap_or(u8::MAX);
        let mut mask = ImageBuffer::from_pixel(self.image.width(), self.image.height(), MASK_TRUE);
        for y in 0..self.image.height() {
            for x in 0..self.image.width() {
                let depth_value = self.depth.get_pixel(x, y).0[0];
                if depth_value < from_parsed || depth_value > to_parsed {
                    *mask.get_pixel_mut(x, y) = MASK_FALSE;
                }
            }
        }
        let mut res = MaskImage {
            image: self.image.clone(),
            mask,
        };
        res.apply_mask();
        res
    }
}
