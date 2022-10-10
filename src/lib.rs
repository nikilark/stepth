#![feature(int_log)]

pub mod depth_image;
pub mod mask_image;
mod helpers;
pub mod operations;

use disage::{self, Position};

#[allow(unused_imports)]
pub use crate::operations::*;

#[allow(unused_imports)]
pub use crate::depth_image::*;

#[allow(unused_imports)]
pub use crate::mask_image::*;

#[allow(unused_imports)]
pub use image;


#[cfg(test)]
mod tests;
