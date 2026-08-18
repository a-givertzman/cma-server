use std::{ops::Range, sync::Arc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::collections::FxIndexMap;
use crate::{err, services::frdm_service::{Bendings, CraneConf, Inputs}};

///
/// ## Evaluation for the crane rope Deprecation
///
/// ### Use [Inputs] to pass a new Event contains a value for the calculation
/// - Expected boom len / angle, rope pos / load events, for example:
///     - [Load.MainBoomAngle], current angle of the boom (relative axis), degrees
///     - [Load.RotaryBoomLen], length of the rotary boom, meter
///     - [Winch.EncoderBR2], current rope position, meter
///     - [Winch.Load], current rope load, tonn
/// - Event mast have proper name, defined in the configured inputs, else it will be ignored
/// - Event mast have value in proper units:
///     - angle: degrees
///     - distances: millimeters
///     - weight: tonn
/// - Event can have type (else it will be ignores):
///     - `Int`
///     - `Real`
///     - `Double`
pub struct Deprecation<'a> {
    inputs: Arc<Inputs>,
    conf: CraneConf,
    /// The length of the each rope segmetn
    /// The entair rope is divided into these segments to calculate Depreciation Rate.
    segment: f64,
    /// The total number of segments of `segment`-length fitting the entire rope.
    slices: usize,
    /// Индексы блоков и слайсов, лежащих на них
    ///                Block     Slices
    blocks: FxIndexMap<usize, Vec<usize>>,
    bendings: Bendings,
    results: Box<dyn Fn(&usize, f64) + 'a>,
    dbg: Dbg,
}
//
//
impl<'a> Deprecation<'a> {
    ///
    /// Returns [Deprecation] new instance
    /// - `inputs` - [Inputs] provides `Events` required for the calculations
    /// - `results` - Callback provides Deprecation results as index of slice and it new Deprecation value
    pub fn new(parent: impl Into<String>, conf: &CraneConf, inputs: Arc<Inputs>, bendings: Bendings, results: impl Fn(&usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "Deprecation");
        inputs.subscribe(conf.rope.load.clone());
        inputs.subscribe(conf.rope.pos.clone());
        Self {
            inputs,
            conf: conf.clone(),
            segment: conf.rope.segment.as_mm(),
            slices: (conf.rope.length.as_mm() / conf.rope.segment.as_mm()).floor() as usize,
            blocks: conf.blocks.iter().enumerate().map(|(i, _)| (i, vec![])).collect(),
            bendings,
            results: Box::new(results),
            dbg,
        }
    }
    ///
    /// Evaluates Boom's values using passed new parameters
    pub fn eval(&mut self) -> Option<()> {
        match self.bendings.eval(&self.inputs) {
            Some(blocks) => {
                // log::debug!("{} | Bendings:", self.dbg);
                // for block in &blocks {
                //     log::debug!("{} | \t Block[{}]: {:.4}..{:.4}", self.dbg, block.name, block.bending.start, block.bending.end);
                // }
                let pos = self.inputs.rope_pos();
                let load = self.inputs.get(&self.conf.rope.load);
                match (pos, load) {
                    (None, None) => {
                        log::warn!("{}.eval | Inputs '{}', '{}' - Not found", self.dbg, self.conf.rope.pos, self.conf.rope.load);
                        return None;
                    }
                    (None, Some(_)) => {
                        log::warn!("{}.eval | Input '{}' - Not found", self.dbg, self.conf.rope.pos);
                        return None;
                    }
                    (Some(_), None) => {
                        log::warn!("{}.eval | Input '{}' - Not found", self.dbg, self.conf.rope.load);
                        return None;
                    }
                    (Some(pos), Some(load)) => {
                        log::debug!("{}.eval | pos {pos} mm,  load {load} tonn", self.dbg);
                        for (block_ix, block) in blocks.iter().enumerate() {
                            // TODO: Расчет износа сознательно упрощен.
                            // И учитывает "количество перегибов каната под нашрузкой".
                            // Но в будущем следует так же учесть и диаметр каната.
                            if !block.skipped {
                                let deprecation = load / (block.diameter * 0.001);
                                match self.slices(&block.bending) {
                                    Err(err) => log::warn!("{}.eval | Block {}: Can't evaluate slices: {:?}", self.dbg, block.name, err),
                                    Ok(current) => {
                                        // Exit: the Slices that are in self.slices but not in current
                                        for slice in &self.blocks[block_ix] {
                                            // log::debug!("{dbg} | Slice[{}] -> Exit ({ix}),  offset: {},  D: {} m,  result: {:?}", self.ix, self.offset, block.diameter * 0.001, result);
                                            if let Err(_) = current.binary_search(slice) {
                                                (self.results)(&slice, deprecation);
                                            }
                                        }
                                        // Enter: the Slices that are in current but not in self.slices
                                        for slice in &current {
                                            if let Err(_) = self.blocks[block_ix].binary_search(slice) {
                                            // log::debug!("{dbg} | Slice[{}] -> Enter ({ix}),  offset: {},  D: {} m,  result: {:?}", self.ix, self.offset, block.diameter * 0.001, result);
                                                (self.results)(&slice, deprecation);
                                            }
                                        }
                                        self.blocks[block_ix] = current;
                                        // log::debug!("{} | Slices: {:?}", self.dbg, self.slices);
                                    }
                                }
                            }
                        }
                        Some(())
                    }
                }
            },
            None => None,
        }
    }
    ///
    /// Returns slices (indexes) sorted ASC, intersects with the bend range
    #[named]
    fn slices(&self, bend: &Range<f64>) -> Result<Vec<usize>, Error> {
        // Количество Слайсов которые приходятся на начало Бенда
        // log::debug!("{}.slices | rope_len: {}", self.dbg, self.rope_len);
        // log::debug!("{}.slices | segment: {}", self.dbg, self.segment);
        // log::debug!("{}.slices | bend: {:?}", self.dbg, bend);
        if !bend.start.is_finite() || bend.start < 0.0 {
            return Err(err!(self.dbg, "Invalid Bend start: {}", bend.start));
        }
        let first_slice = (bend.start / self.segment).trunc() as usize;
        // log::debug!("{}.slices | first_slice: {first_slice}", self.dbg);
        // Точка начала первого Слайса в Бенде
        let start_point = (first_slice as f64) * self.segment;
        // log::debug!("{}.slices | start_point: {start_point}", self.dbg);
        let delta = bend.end - start_point;
        if !delta.is_finite() || delta < 0.0 {
            return Err(err!(self.dbg, "Invalid Bend delta: {}", delta));
        }
        // log::debug!("{}.slices | delta: {delta}", self.dbg);
        let slices = (delta / self.segment).ceil() as usize;    // self.segment - доверяем этому значению, проверяется в конфиге
        let end = first_slice.saturating_add(slices);
        if end > self.slices {
            return Err(err!(self.dbg, "Invalid slice range: end {} exceeds maximum {}", slices, self.slices));
        }
        // log::debug!("{}.slices | slices: {slices}", self.dbg);
        Ok(Vec::from_iter(first_slice..end))
    }
}
///
/// Testing such functionality / behavior
#[cfg(test)]
#[test]
fn test_slices() {
    use std::{sync::atomic::AtomicBool, time::{Duration, Instant}};
    use sal_sync::{services::{conf::{ConfTree, ServicesConf}, Services}, thread_pool::ThreadPool};
    use testing::stuff::max_test_duration::TestDuration;
    use crate::services::frdm_service::{BlockArcs, Blocks, Booms, FrdmServiceConf, RopeSections};
    env_logger::Builder::new().filter_level(log::LevelFilter::Debug).init();
    log::debug!("");
    let dbg = Dbg::own("Deprecation.slices");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data: &[(i32, Range<f64>, Vec<usize>)] = &[
        (01,  0.0.. 5.0, vec![0]),
        (02,  0.0..10.0, vec![0]),

        (10, 08.0..10.0, vec![0]),
        (11, 09.0..11.0, vec![0, 1]),
        (12, 10.0..12.0, vec![1]),
        (13, 11.0..13.0, vec![1]),

        (30, 11.0..35.0, vec![1, 2, 3]),
        (31, 12.0..35.0, vec![1, 2, 3]),
        (32, 15.0..35.0, vec![1, 2, 3]),
        (33, 17.0..35.0, vec![1, 2, 3]),
        (34, 19.0..35.0, vec![1, 2, 3]),

        (41, 15.0..31.0, vec![1, 2, 3]),
        (42, 15.0..32.0, vec![1, 2, 3]),
        (43, 15.0..35.0, vec![1, 2, 3]),
        (44, 15.0..37.0, vec![1, 2, 3]),
        (45, 15.0..39.0, vec![1, 2, 3]),

        (51, 15.0..39.0, vec![1, 2, 3]),
        (52, 15.0..40.0, vec![1, 2, 3]),
        (53, 15.0..41.0, vec![1, 2, 3, 4]),

    ];
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        rope:
            width: 35 mm
            length: 1 m
            aux-length: 1 m
            segment: 10 mm
            pos: point real 'Winch.Pos'
            load: point real 'Winch.Load'
        booms:
            - Main-Boom:
                l1: 0.0 mm
                l2: 0.0 mm
                l3: 0.0 mm
                l4: 10330.0 mm
                len: 11200.0 mm
                angle: point real 'MainBoom.Angle'
                parking: 0.0 deg
        blocks:
            - 1:
                lf: 1830.0 mm,  710.0 mm
                d: 845.670 mm
                scheme: TopTop
                bind: Drum
    ").unwrap());

    let crane_conf = CraneConf::new(&dbg, conf);
    let mut conf = FrdmServiceConf::default();
    conf.rope_deprecation.crane = crane_conf;
    let tp = ThreadPool::new(&dbg, Some(4));
    let services = Arc::new(Services::new(
        &dbg,
        ServicesConf::new(&dbg, ConfTree::new_root(serde_yaml::from_str(r"").unwrap())),
        Some(tp.scheduler()),
    ).unwrap());
    let exit = Arc::new(AtomicBool::new(false));
    let inputs = Arc::new(Inputs::new(&dbg, &conf, services, tp.scheduler(), exit));
    let parking = false;
    let deprecation = Deprecation::new(
        &dbg,
        &conf.rope_deprecation.crane,
        inputs.clone(),
        Bendings::new(
            &dbg,
            &conf.rope_deprecation.crane.rope,
            BlockArcs::new(
                &dbg,
                &conf.rope_deprecation.crane.rope.segment,
                RopeSections::new(
                    &dbg,
                    Blocks::new(
                        &dbg,
                        conf.rope_deprecation.crane.rope.aux_length,
                        &conf.rope_deprecation.crane.blocks,
                        parking,
                        Booms::new(&dbg, &conf.rope_deprecation.crane.booms, inputs, parking),
                    ),
                ),
            ),
        ),
        |_, _| {},
    );
    let mut t;
    for (step, bendings, target) in test_data {
        t = Instant::now();
        let result = deprecation.slices(&bendings).unwrap();
        log::debug!("{dbg} | {step}  result: {:?}, target: {:?},  elapsed: {:?}", result, target, t.elapsed());
        assert!(result == target.to_owned(), "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
