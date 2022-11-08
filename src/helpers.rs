use super::*;
use disage::{pixels::PixelOpps, Position};

// pub fn relative_pos(pos: Position, size: Dimensions, size_to: Dimensions) -> Position {
//     let (rx, ry) = (
//         (pos.x as f64 / size.width as f64),
//         (pos.y as f64 / size.height as f64),
//     );
//     Position {
//         x: (size_to.width as f64 * rx) as u32,
//         y: (size_to.height as f64 * ry) as u32,
//     }
// }

pub fn distance_dot_dot(f: Position, s: Position) -> u32 {
    let (x1, y1) = (f.x as i64, f.y as i64);
    let (x2, y2) = (s.x as i64, s.y as i64);
    (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f64).sqrt() as u32
}

pub fn distance_dot_array<T: Clone + PixelOpps<T>>(
    what: &T,
    array: &Vec<Vec<T>>,
    from: Position,
    max: u32,
    precision: T,
) -> Option<(u32, Position)> {
    let (x, y): (u32, u32) = from.tuplexy();
    let x = x as i64;
    let y = y as i64;
    let get2d = |array: &Vec<Vec<T>>, i: i64, j: i64| match array.get(i as usize) {
        Some(t) => match t.get(j as usize) {
            Some(v) => Some(v.clone()),
            None => None,
        },
        None => None,
    };
    for current_step in 0..max as i64 {
        let mut still_in_bounds = false;
        for (main, sub, order) in [(y, x, true), (x, y, false)] {
            for i in [main + current_step, main - current_step] {
                for j in sub - current_step..sub + current_step + 1 {
                    let (point_y, point_x) = if order { (i, j) } else { (j, i) };
                    match get2d(&array, point_y, point_x) {
                        Some(v) => {
                            still_in_bounds = true;
                            if v.clone().substract(what.clone()).lt(precision.clone()) {
                                let pos = Position::new(point_x as u32, point_y as u32);
                                let distance = distance_dot_dot(
                                    from,
                                    pos
                                );
                                return Some((distance, pos));
                            }
                        }
                        None => continue,
                    }
                }
            }
        }
        if !still_in_bounds {
            break;
        }
    }
    None
}
