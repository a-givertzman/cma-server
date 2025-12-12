mod bendings;
mod block_arcs;
mod block;
mod blocks;
mod boom;
mod booms;
mod input_kind;
mod deprecation;
mod offset;
mod rope_sections;

pub(crate) use bendings::*;
pub(crate) use block_arcs::*;
pub(crate) use block::*;
pub(crate) use blocks::*;
pub(crate) use boom::*;
pub(crate) use booms::*;
pub(crate) use input_kind::*;
pub(crate) use deprecation::*;
pub(crate) use offset::*;
pub(crate) use rope_sections::*;

///
/// 
pub(super) fn rotate_xy(lx: f64, ly: f64, alpha: f64) -> Offset<f64> {
    let angle_rad = alpha.to_radians();
    Offset::new(lx * angle_rad.cos() - ly * angle_rad.sin(), lx * angle_rad.sin() + ly * angle_rad.cos())
}
