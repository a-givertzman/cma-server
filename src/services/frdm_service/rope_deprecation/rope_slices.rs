use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, CraneConf, Deprication, LooseRopeSections, RopeSlice};

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
        let mut subscriptions = vec![];
        Self {
            eval: Deprication::new(
                &dbg,
                &conf,
                Bendings::new(
                    &dbg,
                    conf.rope.pos,
                    conf.rope.winch_len,
                    BlockArcs::new(
                        &dbg,
                        LooseRopeSections::new(
                            &dbg,
                            Blocks::new(
                                &dbg,
                                &conf.blocks,
                                Booms::new(&dbg, &conf.booms, &mut subscriptions),
                            ),
                        ),
                    ),
                ),
                subscriptions,
                |index, deprication| {

                },
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
    /// Returns current calue from inputs by the key if exists
    pub fn get(&self, key: &str) -> Option<f64> {
        self.eval.get(key)
    }
    ///
    /// Evaluates rope slices deprication depend on boom len / angle, rope pos / load event was received,
    /// New deprecation result can be evaluated and passed via `deprication` callback
    pub fn eval(&mut self, event: &Point) {
        match self.eval.eval(event) {
            Some(_) => todo!(),
            None => todo!(),
        };
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