use super::{disage, helpers, image};

pub struct DepthHasher<T: disage::pixels::PixelOpps<T>> {
    additional_img: Vec<Vec<T>>,
    add_img_size: disage::Dimensions,
    main_img_size: disage::Dimensions,
    precision: T,
}

impl<P: image::Primitive + 'static> DepthHasher<[P; 3]>
where
    [P; 3]: disage::pixels::PixelOpps<[P; 3]>,
{
    pub fn from_rgb(
        add_img: &image::ImageBuffer<image::Rgb<P>, Vec<P>>,
        main_img_size: disage::Dimensions,
        precision: [P; 3],
    ) -> Self {
        DepthHasher {
            additional_img: disage::converters::pixels_to_array(
                &disage::converters::raw_rgb(add_img),
                add_img.width(),
            ),
            add_img_size: disage::Dimensions::new(add_img.height(), add_img.width()),
            main_img_size,
            precision,
        }
    }
}

impl<P: image::Primitive + 'static + disage::pixels::PixelOpps<P>> DepthHasher<P> {
    pub fn from_luma(
        add_img: &image::ImageBuffer<image::Luma<P>, Vec<P>>,
        main_img_size: disage::Dimensions,
        precision: P,
    ) -> Self {
        DepthHasher {
            additional_img: disage::converters::pixels_to_array(
                &disage::converters::raw_luma(add_img),
                add_img.width(),
            ),
            add_img_size: disage::Dimensions::new(add_img.height(), add_img.width()),
            main_img_size,
            precision,
        }
    }
}

impl<P: disage::pixels::PixelOpps<P> + Clone> disage::hashers::PixelHasher<P, u32>
    for DepthHasher<P>
{
    fn hash(&self, data: &[Vec<P>], position: disage::Position, _size: disage::Dimensions) -> u32 {
        let rel_pos = helpers::relative_pos(position, self.main_img_size, self.add_img_size);
        let max_side = self.add_img_size.width.max(self.add_img_size.height);
        let max_dist = max_side / 20;
        helpers::distance_dot_array(
            &data[position.y as usize][position.x as usize],
            &self.additional_img,
            rel_pos,
            max_dist,
            self.precision.clone(),
        )
        .unwrap_or(0)
    }
}

pub struct DepthChecker<T> {
    pub precision: T,
}

impl disage::checkers::PixelEqChecker<u32> for DepthChecker<u32> {
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
