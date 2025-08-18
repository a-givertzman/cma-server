use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::Point};
use crate::services::frdm_service::{Blocks, Booms, CraneConf, Deprication, RopeSlice};

///
/// The collection of [RopeSlice]
/// - Devide rope by specified in the config number of slices
/// - Calculate deprecation for each slice
pub struct RopeSlices<'a> {
    eval: Deprication,
    slices: Vec<RopeSlice>,
    conf: CraneConf,
    deprecation: Box<dyn Fn(usize, f64) + 'a>,
    dbg: Dbg,
}
//
//
impl<'a> RopeSlices<'a> {
    ///
    /// Returns [RopeSlices] new instance
    /// - `deprecation` - Here will be passed evaluated deprecation for each [RopeSlice] with it's index,
    pub fn new(parent: impl Into<String>, conf: CraneConf, deprecation: impl Fn(usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "RopeSlices");
        let slices = (conf.rope.length.as_m() / conf.rope.segment.as_m()).ceil() as usize;
        log::debug!("{dbg}.new | Rope: {} m, slices: {slices}, devided by {:.2} mm", conf.rope.length.as_m(), conf.rope.segment.as_mm());
        let mut inputs = FxIndexMap::default();
        Self {
            eval: Deprication::new(
                &dbg,
                &conf.rope,
                Blocks::new(
                    &dbg,
                    &conf.blocks,
                    Booms::new(&dbg, &conf.booms, &mut inputs),
                ),
                inputs,
            ),
            slices: (0..slices).map(|slice| {
                let offset = (slice as f64) * conf.rope.segment.as_m();
                log::trace!("{dbg}.new | Slice: {slice}: offset: {:.2}", offset);
                RopeSlice::new(slice, &conf.bendings, offset)
            }).collect(),
            conf,
            deprecation: Box::new(deprecation),
            dbg,
        }
    }
    
    ///
    /// Evaluates rope slices deprication depend on boom len / angle, rope pos / load event was received,
    /// New deprecation result can be evaluated and passed via `deprication` callback
    pub fn eval(&mut self, event: &Point) {
        match self.eval.eval(event) {
            Some(blocks) => todo!(),
            None => todo!(),
        };
        match event.name() {
            // name if name == conf.crane.rope.pos => {
            //     let pos = point.to_double().as_double().value;
            //     log::debug!("{dbg}.run | Received rope pos: {:.4?} m", pos);
            //     rope_pos.store((pos * 1000.0).round() as usize, Ordering::SeqCst);
            //     rope_pos_ok.store(true, Ordering::SeqCst);
            //     rope_slices.eval(Some(pos), None);
            // }
            // name if name == conf.crane.rope.load => {
            //     let load = point.to_double().as_double().value;
            //     log::debug!("{dbg}.run | Received rope load: {:.4?} tonn", load);
            //     rope_slices.eval(None, Some(load));
            // }
            // name if name == conf.crane.booms.main_angle => {
            //     let main_angle = point.to_double().as_double().value;
            //     log::debug!("{dbg}.run | Received boom.main_angle: {:.4?}", main_angle);
            // }
            // name if name == conf.crane.booms.rotary_angle => {
            //     let rotary_angle = point.to_double().as_double().value;
            //     log::debug!("{dbg}.run | Received boom.rotary_angle: {:.4?}", rotary_angle);
            // }
            _ => log::warn!("{}.run | Unknown point name: {:?}", self.dbg, event.name()),
        }

        // match (pos, load) {
        //     (None, None) => {},
        //     (None, Some(load)) => for slice in &mut self.slices { slice.add_load(load) },
        //     (Some(pos), None) => for slice in &mut self.slices { slice.add_pos(pos) },
        //     (Some(pos), Some(load)) => {
        //         for slice in &mut self.slices {
        //             slice.add_pos(pos);
        //             slice.add_load(load);
        //         }
        //     }
        // }
        for slice in &mut self.slices {
            if let Some(deprecation) = slice.deprecation(&self.conf.bendings, todo!(), todo!()) {
                (self.deprecation)(slice.id(), deprecation);
            }
        }
    }
}