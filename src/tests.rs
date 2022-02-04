use super::*;

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
