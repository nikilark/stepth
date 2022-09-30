use image::{ImageBuffer};


#[derive(Clone)]
pub struct DepthImage {
    pixels: ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    depth: ImageBuffer<image::Luma<u8>, Vec<u8>>,
}

impl DepthImage {
    pub fn open(main_path: &str, depth_path: &str) -> Self {
        let pixels = image::open(main_path).unwrap().to_rgb8();
        let depth = image::open(depth_path).unwrap().to_luma8();
        DepthImage { pixels, depth }
    }

    pub fn save(&self, image_path: &str, depth_path: &str) -> () {
        self.depth.save(depth_path).unwrap();
        self.pixels.save(image_path).unwrap();
    }

    pub fn slice(&self, from: Option<u8>, to: Option<u8>) -> Self {
        let from_parsed = from.unwrap_or(0);
        let to_parsed = to.unwrap_or(u8::MAX);
        let mut cloned = self.clone();
        cloned
            .pixels
            .pixels_mut()
            .zip(cloned.depth.pixels_mut())
            .for_each(|(p, d)| {
                if from_parsed >= d.0[0] || d.0[0] >= to_parsed {
                    p.0 = [0; 3];
                    d.0 = [0];
                }
            });
        cloned
    }
}
