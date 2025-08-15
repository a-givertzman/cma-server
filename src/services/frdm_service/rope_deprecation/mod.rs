mod bendings_conf;
mod block_conf;
mod block;
mod blocks;
mod boom_conf;
mod boom;
mod booms;
mod crane_conf;
mod input_kind;
mod offset;
mod rope_conf;
mod rope_deprecation_conf;
mod rope_deprecation;
mod rope_slice;
mod rope_slices;

pub(crate) use bendings_conf::*;
pub(crate) use block_conf::*;
pub(crate) use block::*;
pub(crate) use blocks::*;
pub(crate) use boom_conf::*;
pub(crate) use boom::*;
pub(crate) use booms::*;
pub(crate) use crane_conf::*;
pub(crate) use input_kind::*;
pub(crate) use offset::*;
pub(crate) use rope_conf::*;
pub(crate) use rope_deprecation_conf::*;
pub(crate) use rope_deprecation::*;
pub(crate) use rope_slice::*;
pub(crate) use rope_slices::*;

///
/// 
pub(super) fn rotate_xy(lx: f64, ly: f64, alpha: f64) -> Offset<f64> {
    let angle_rad = alpha.to_radians();
    Offset::new(lx * angle_rad.cos() - ly * angle_rad.sin(), lx * angle_rad.sin() + ly * angle_rad.cos())
}
