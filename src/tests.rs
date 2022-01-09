use super::*;

#[test]
fn precision_test() {
    let mut img_rgb: ImageBuffer<Rgb<u16>, Vec<u16>> = image::ImageBuffer::new(3, 3);
    img_rgb
        .pixels_mut()
        .enumerate()
        .for_each(|(i, v)| *v = Rgb([i as u16, i as u16, i as u16]));
    let mut img_luma: ImageBuffer<Luma<u16>, Vec<u16>> = image::ImageBuffer::new(3, 3);
    img_luma
        .pixels_mut()
        .enumerate()
        .for_each(|(i, v)| *v = Luma([i as u16]));
    assert_eq!(helpers::precision_rgb16(&img_rgb, 0.5), [1, 1, 1]);
    assert_eq!(helpers::precision_luma16(&img_luma, 0.5), 4);
}

#[test]
fn relative_pos_test() {
    assert_eq!(
        helpers::relative_pos(
            Position::new(5, 5),
            Dimensions::new(10, 10),
            Dimensions::new(20, 20)
        ),
        Position::new(10, 10)
    );
}

#[test]
fn distance_dot_dot_test() {
    assert_eq!(
        helpers::distance_dot_dot(Position::new(0, 0), Position::new(1, 0)),
        1
    );
    assert_eq!(
        helpers::distance_dot_dot(Position::new(0, 0), Position::new(1, 1)),
        1
    );
    assert_eq!(
        helpers::distance_dot_dot(Position::new(0, 0), Position::new(2, 2)),
        2
    );
    assert_eq!(
        helpers::distance_dot_dot(Position::new(0, 0), Position::new(2, 3)),
        3
    );
}

#[test]
fn distance_dot_array_test() {
    let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
    assert_eq!(
        helpers::distance_dot_array(&0, &arr, Position::new(0, 0), 10, 1),
        Some(0)
    );
    assert_eq!(
        helpers::distance_dot_array(&0, &arr, Position::new(1, 1), 10, 1),
        Some(1)
    );
    assert_eq!(
        helpers::distance_dot_array(&0, &arr, Position::new(2, 2), 10, 1),
        Some(2)
    );
    assert_eq!(
        helpers::distance_dot_array(&0, &arr, Position::new(2, 2), 1, 1),
        None
    );
    assert_eq!(
        helpers::distance_dot_array(&0, &arr, Position::new(22, 22), 100, 1),
        None
    );
}

#[test]
fn distance_discrete_pixels_test() {
    let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
    let pix1 = DiscretePixel::new(0u16, Position::new(0, 0), Dimensions::new(100, 100));
    let pix2 = DiscretePixel::new(0u16, Position::new(10, 0), Dimensions::new(100, 100));
    let pix3 = DiscretePixel::new(5u16, Position::new(20, 20), Dimensions::new(100, 100));
    let pix4 = DiscretePixel::new(2u16, Position::new(100, 100), Dimensions::new(100, 100));
    let vpix = vec![pix1.clone(), pix2.clone(), pix3.clone(), pix4.clone()];
    let pos_res = vec![
        (pix1, Some(0u32)),
        (pix2, Some(1)),
        (pix3, Some(1)),
        (pix4, None),
    ];
    assert_eq!(
        helpers::distance_discrete_pixels(&vpix, Dimensions::new(30, 30), &arr, 5, 1,1),
        pos_res
    );
}

#[test]
fn dpixels_to_array_test() {
    let arr = vec![vec![0u16, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
    let pix1 = DiscretePixel::new(0u16, Position::new(0, 0), Dimensions::new(4, 2));
    let pix2 = DiscretePixel::new(0u16, Position::new(2, 0), Dimensions::new(2, 2));
    let pix3 = DiscretePixel::new(10u16, Position::new(2, 2), Dimensions::new(2, 2));
    let v = vec![
        (pix1.clone(), Some(1u32)),
        (pix2.clone(), Some(2)),
        (pix3.clone(), None),
    ];
    let pos_res = vec![
        vec![Some(1u32), Some(1), Some(2), Some(2)],
        vec![Some(1), Some(1), Some(2), Some(2)],
        vec![Some(1), Some(1), None, None],
        vec![Some(1), Some(1), None, None],
    ];
    assert_eq!(
        helpers::distance_discrete_pixels(
            &vec![pix1, pix2, pix3],
            Dimensions::new(3, 3),
            &arr,
            3,
            1,
            0
        ),
        v.clone()
    );
    assert_eq!(helpers::depthp_to_array(&v, Dimensions::new(4, 4)), pos_res);
}
