use image::{DynamicImage, ImageBuffer, Luma};

pub const MASK_TRUE: Luma<u8> = Luma([u8::MAX; 1]);
pub const MASK_FALSE: Luma<u8> = Luma([u8::MIN; 1]);

#[derive(Clone, Debug, Default)]
pub struct MaskImage {
    pub image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub mask: ImageBuffer<image::Luma<u8>, Vec<u8>>,
}

impl MaskImage {
    pub fn open(image_path: &str) -> Self {
        MaskImage::from_image(image::open(image_path).unwrap())
    }

    pub fn from_image(img: DynamicImage) -> Self {
        let image = img.to_rgba8();
        let mask = ImageBuffer::from_pixel(image.width(), image.height(), MASK_TRUE);
        MaskImage { image, mask }
    }

    pub fn image(&self) -> image::DynamicImage {
        image::DynamicImage::ImageRgba8(self.image.clone())
    }

    pub fn mask(&self) -> image::DynamicImage {
        image::DynamicImage::ImageLuma8(self.mask.clone())
    }

    pub fn load_mask(
        &mut self,
        mask: ImageBuffer<image::Luma<u8>, Vec<u8>>,
    ) -> Result<(), std::io::Error> {
        if mask.width() == self.width() && mask.height() == self.height() {
            self.mask = mask;
            return Ok(());
        } else {
            self.mask = DynamicImage::ImageLuma8(mask)
                .resize(self.width(), self.height(), image::imageops::Gaussian)
                .to_luma8();
            return Ok(());
        }
    }

    pub fn load_mask_from_file(&mut self, mask_path: &str) -> Result<(), std::io::Error> {
        let mask_image = image::open(mask_path);
        if mask_image.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to open image",
            ));
        }
        self.load_mask(mask_image.unwrap().to_luma8())
    }

    pub fn highlight_mask(&self) -> DynamicImage {
        let mut res = self.image.clone();
        res.pixels_mut().zip(self.mask.pixels()).for_each(|(p, d)| {
            if *d == MASK_TRUE {
                let multiplier = 2.0;
                let adjust = |v: u8, pos: bool| {
                    (v as f32 * if pos { multiplier } else { 1.0 / multiplier })
                        .max(0.0)
                        .min(255.0) as u8
                };
                p.0[0] = adjust(p.0[0], true);
                p.0[1] = adjust(p.0[1], false);
                p.0[2] = adjust(p.0[2], false);
            }
        });
        DynamicImage::ImageRgba8(res)
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
        self.mask = DynamicImage::ImageLuma8(self.mask.clone())
            .resize(to.width, to.height, image::imageops::Gaussian)
            .to_luma8();
    }

    pub fn dimensions(&self) -> disage::Dimensions {
        disage::Dimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    pub fn image_replace(&mut self, other: &MaskImage, start_point: disage::Position) {
        let (start_x, start_y) = start_point.tuplexy();
        for y in start_y..(start_y + other.height()).min(self.height()) {
            for x in start_x..(start_x + other.width()).min(self.width()) {
                if *self.mask.get_pixel(x, y) != MASK_TRUE {
                    continue;
                }
                self.image.put_pixel(x, y, *other.image.get_pixel(x, y));
            }
        }
    }

    pub fn image_brightness(&mut self, value: i32) {
        let image_to_mod = DynamicImage::ImageRgba8(self.image.clone()).brighten(value);
        self.image_replace(
            &MaskImage::from_image(image_to_mod),
            disage::Position::new(0, 0),
        );
    }

    pub fn image_contrast(&mut self, value: i32) {
        let image_to_mod = DynamicImage::ImageRgba8(self.image.clone()).adjust_contrast(value as f32);
        self.image_replace(
            &MaskImage::from_image(image_to_mod),
            disage::Position::new(0, 0),
        );
    }

    pub fn image_sharpness(&mut self, value: i32) {
        let image_to_mod = DynamicImage::ImageRgba8(self.image.clone()).unsharpen(value as f32, 20);
        self.image_replace(
            &MaskImage::from_image(image_to_mod),
            disage::Position::new(0, 0),
        );
    }

    pub fn mask_copy(&mut self, other: &MaskImage) {
        self.load_mask(other.mask.clone()).unwrap()
    }

    pub fn mask_and(&mut self, other: &MaskImage) {
        let (height, width) = self.dimensions().tuplehw();
        self.mask
            .pixels_mut()
            .zip(
                if disage::Dimensions::from_tuplehw((height, width)) != other.dimensions() {
                    DynamicImage::ImageLuma8(other.mask.clone())
                        .resize(width, height, image::imageops::FilterType::Gaussian)
                        .to_luma8()
                } else {
                    other.mask.clone()
                }
                .pixels(),
            )
            .for_each(|(pix, other_pix)| {
                *pix = if *pix == MASK_TRUE && *other_pix == MASK_TRUE {
                    MASK_TRUE
                } else {
                    MASK_FALSE
                };
            });
    }

    pub fn mask_or(&mut self, other: &MaskImage) {
        let (height, width) = self.dimensions().tuplehw();
        self.mask
            .pixels_mut()
            .zip(
                if disage::Dimensions::from_tuplehw((height, width)) != other.dimensions() {
                    DynamicImage::ImageLuma8(other.mask.clone())
                        .resize(width, height, image::imageops::FilterType::Gaussian)
                        .to_luma8()
                } else {
                    other.mask.clone()
                }
                .pixels(),
            )
            .for_each(|(pix, other_pix)| {
                *pix = if *pix == MASK_TRUE || *other_pix == MASK_TRUE {
                    MASK_TRUE
                } else {
                    MASK_FALSE
                };
            });
    }

    pub fn mask_not(&mut self) {
        self.mask.pixels_mut().for_each(|p| p.0[0] = 255 - p.0[0]);
    }

    pub fn save(&self, path: &str) -> () {
        self.image.save(path).unwrap();
    }

    pub fn mask_reset(&mut self) {
        self.mask = ImageBuffer::from_pixel(self.width(), self.height(), MASK_TRUE);
    }

    pub fn apply_mask(&mut self) -> () {
        for y in 0..self.image.height() {
            for x in 0..self.image.width() {
                if *self.mask.get_pixel(x, y) == MASK_FALSE {
                    self.image.get_pixel_mut(x, y).0 = [0; 4];
                }
            }
        }
    }
}
