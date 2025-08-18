mod block;
mod blocks;
mod boom;
mod booms;
mod input_kind;
mod deprication;
mod offset;

pub(crate) use block::*;
pub(crate) use blocks::*;
pub(crate) use boom::*;
pub(crate) use booms::*;
pub(crate) use input_kind::*;
pub(crate) use deprication::*;
pub(crate) use offset::*;

///
/// 
pub(super) fn rotate_xy(lx: f64, ly: f64, alpha: f64) -> Offset<f64> {
    let angle_rad = alpha.to_radians();
    Offset::new(lx * angle_rad.cos() - ly * angle_rad.sin(), lx * angle_rad.sin() + ly * angle_rad.cos())
}
