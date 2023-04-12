use crate::{helpers, mask_image::*};
use image::{imageops, DynamicImage, ImageBuffer, Luma};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct DepthImage {
    pub image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub depth: ImageBuffer<image::Luma<u8>, Vec<u8>>,
}

impl DepthImage {
    pub fn open(image_path: &str) -> Result<Self, std::io::Error> {
        let image = image::open(image_path)
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Failed to open image")
            })?
            .to_rgba8();
        let depth = ImageBuffer::from_pixel(image.width(), image.height(), Luma([0u8]));
        Ok(DepthImage { image, depth })
    }

    pub fn from_image(img: DynamicImage) -> Self {
        let image = img.to_rgba8();
        let depth = ImageBuffer::from_pixel(image.width(), image.height(), Luma([0u8]));
        DepthImage { image, depth }
    }

    pub fn image(&self) -> image::DynamicImage {
        image::DynamicImage::ImageRgba8(self.image.clone())
    }

    pub fn depth(&self) -> image::DynamicImage {
        image::DynamicImage::ImageLuma8(self.depth.clone())
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

    pub fn highlight_depth(&self) -> DynamicImage {
        let mut res = self.image.clone();
        res.pixels_mut()
            .zip(self.depth.pixels())
            .for_each(|(p, d)| {
                let multiplier = d.0[0] as f32 / 255.0 * 2.0;
                let adjust = |v: u8| (v as f32 * multiplier).max(0.0).min(255.0) as u8;
                p.0[0] = adjust(p.0[0]);
                p.0[1] = adjust(p.0[1]);
                p.0[2] = adjust(p.0[2]);
            });
        DynamicImage::ImageRgba8(res)
    }

    pub fn open_depth(&mut self, depth_path: &str) -> Result<(), std::io::Error> {
        let depth_image = image::open(depth_path);
        if depth_image.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to open image",
            ));
        }
        self.load_depth(depth_image.unwrap().to_luma8())
    }

    pub fn open_depth_from_additional(
        &mut self,
        add_path: &str,
        precision: [u8; 3],
    ) -> Result<(), std::io::Error> {
        let add_image = image::open(add_path);
        if add_image.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to open image",
            ));
        }
        self.load_depth_from_additional(add_image.unwrap(), precision)
    }

    pub fn load_depth_from_additional(
        &mut self,
        add_image: image::DynamicImage,
        precision: [u8; 3],
    ) -> Result<(), std::io::Error> {
        let add_image = add_image.to_rgb8();
        let add_array = disage::converters::pixels_to_array(
            &disage::converters::raw_rgb(&add_image),
            add_image.width(),
        );
        let pix_count = (self.width() * self.height()) as f32;
        let min_splits = 16 as usize;
        let max_splits = pix_count.log2().ceil() as usize;
        let mut discr_main = disage::open::rgb_discrete(
            &image::DynamicImage::ImageRgba8(self.image.clone()).to_rgb8(),
            disage::hashers::MeanBrightnessHasher {},
            disage::checkers::BrightnessChecker { precision },
            (min_splits, max_splits),
        );
        let mut pixels: Vec<disage::DiscretePixel<&mut [u8; 3]>> = discr_main.pixels_mut();
        let chunk_size = pixels.len() / 8;
        pixels.par_chunks_mut(chunk_size).for_each(|v| {
            v.iter_mut().for_each(|p| {
                let middle = disage::Position::new(
                    (p.position.x + p.size.width) / 2,
                    (p.position.y + p.size.height) / 2,
                );
                let (d, _) =
                    helpers::distance_dot_array(p.value, &add_array, middle, 255, precision)
                        .unwrap_or((u32::MIN, disage::Position::new(0, 0)));
                *p.value = [d as u8; 3]
            })
        });
        let max = pixels.iter().max_by_key(|p| p.value[0]).unwrap().value[0];
        pixels.par_chunks_mut(chunk_size).for_each(|v| {
            v.iter_mut().for_each(|p| {
                *p.value = [(p.value[0] as u64 * u8::MAX as u64 / max as u64) as u8; 3]
            })
        });
        let depth_image = DynamicImage::ImageLuma8(disage::converters::to_luma8_from_rgb8(
            &discr_main.collect(),
        ))
        .resize(self.width(), self.height(), imageops::Gaussian)
        .to_luma8();
        self.load_depth(depth_image)
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
                .map(|(_, v)| (v.iter().map(|v| *v as usize).sum::<usize>() / v.len().max(1)) as u8)
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
        let img_min = self.depth.as_raw().iter().min().unwrap().clone();
        let img_max = self.depth.as_raw().iter().max().unwrap().clone();
        let init_centers = (img_min..img_max)
            .step_by(((img_max - img_min) / (zones - 1)) as usize - 1)
            .collect();
        inner(self.depth.as_raw(), init_centers)
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
        MaskImage {
            image: self.image.clone(),
            mask,
        }
    }
}
