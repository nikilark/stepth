use super::{disage, helpers, image};

pub struct DepthHasher<T: disage::pixels::PixelOpps<T>> {
    additional_img: Vec<Vec<T>>,
    add_img_size: disage::Dimensions,
    main_img_size: disage::Dimensions,
    precision: T,
}

impl DepthHasher<[u16; 3]> {
    pub fn from_additional(
        add_img: &image::ImageBuffer<image::Rgb<u16>, Vec<u16>>,
        main_img_size: disage::Dimensions,
        precision: [u16; 3],
    ) -> Self {
        let pixels: Vec<image::Rgb<u16>> = add_img.pixels().map(|f| f.clone()).collect();
        DepthHasher {
            additional_img: disage::DiscreteImage::<u8>::pixels_to_array(&pixels, add_img.width()),
            add_img_size: disage::Dimensions::new(add_img.height(), add_img.width()),
            main_img_size,
            precision,
        }
    }
}

impl disage::hashers::PixelHasher<[u16; 3], u32> for DepthHasher<[u16; 3]> {
    fn hash(
        &self,
        data: &[Vec<[u16; 3]>],
        position: disage::Position,
        _size: disage::Dimensions,
    ) -> u32 {
        let rel_pos = helpers::relative_pos(position, self.main_img_size, self.add_img_size);
        let max_side = self.add_img_size.width.max(self.add_img_size.height);
        let max_dist = max_side / 20;
        helpers::distance_dot_array(
            &data[position.y as usize][position.x as usize],
            &self.additional_img,
            rel_pos,
            max_dist,
            self.precision,
        )
        .unwrap_or(0)
    }
}

pub struct DepthChecker<T> {
    pub precision: T,
}

impl disage::hashers::PixelEqChecker<u32> for DepthChecker<u32> {
    fn eq(&self, left: u32, right: u32) -> bool {
        if left > 0 && right > 0 {
            if left.max(right) - left.min(right) > self.precision {
                false
            } else {
                true
            }
        } else {
            true
        }
    }
}
