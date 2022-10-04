use image::{DynamicImage, ImageBuffer, Luma};

pub const MASK_TRUE: Luma<u8> = Luma([u8::MAX; 1]);
pub const MASK_FALSE: Luma<u8> = Luma([u8::MIN; 1]);

#[derive(Clone)]
pub struct MaskImage {
    pub image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    pub mask: ImageBuffer<image::Luma<u8>, Vec<u8>>,
}

impl MaskImage {
    pub fn open(image_path: &str) -> Self {
        let image = image::open(image_path).unwrap().to_rgba8();
        let mask = ImageBuffer::from_pixel(image.width(), image.height(), MASK_TRUE);
        MaskImage { image, mask }
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
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unknown error",
        ))
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
        self.image
            .pixels_mut()
            .zip(self.mask.pixels())
            .for_each(|(pix, m)| {
                if *m == MASK_TRUE {
                    let transform = |pix: u8| {
                        let raw_value = pix as i32 + value;
                        raw_value.min(u8::MAX.into()).max(u8::MIN.into()) as u8
                    };
                    pix.0 = [
                        transform(pix.0[0]),
                        transform(pix.0[1]),
                        transform(pix.0[2]),
                        pix.0[3],
                    ];
                }
            });
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
