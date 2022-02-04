use image::ImageBuffer;

use crate::depth_hasher;

pub fn luma_depth<
    P: image::Primitive
        + 'static
        + disage::pixels::PixelOpps<P>
        + std::marker::Send
        + std::marker::Sync,
>(
    main_img: &ImageBuffer<image::Luma<P>, Vec<P>>,
    additional_img: &ImageBuffer<image::Luma<P>, Vec<P>>,
    depth_prec: u32,
    pixel_prec: P,
    splits: (usize, usize),
) -> disage::DiscreteImage<u32> {
    let equality_checker = depth_hasher::DepthChecker {
        precision: depth_prec,
    };
    let hasher = depth_hasher::DepthHasher::from_luma(
        &additional_img,
        disage::Dimensions::new(main_img.height(), main_img.width()),
        pixel_prec.clone(),
    );
    disage::open::luma_discrete(main_img, hasher, equality_checker, splits)
}

pub fn rgb_depth<P: image::Primitive + 'static + std::marker::Send + std::marker::Sync>(
    main_img: &ImageBuffer<image::Rgb<P>, Vec<P>>,
    additional_img: &ImageBuffer<image::Rgb<P>, Vec<P>>,
    depth_prec: u32,
    pixel_prec: [P; 3],
    splits: (usize, usize),
) -> disage::DiscreteImage<u32>
where
    [P; 3]: disage::pixels::PixelOpps<[P; 3]>,
{
    let equality_checker = depth_hasher::DepthChecker {
        precision: depth_prec,
    };
    let hasher = depth_hasher::DepthHasher::from_rgb(
        &additional_img,
        disage::Dimensions::new(main_img.height(), main_img.width()),
        pixel_prec.clone(),
    );
    disage::open::rgb_discrete(main_img, hasher, equality_checker, splits)
}
