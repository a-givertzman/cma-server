mod rope_deprecation {
mod algorithm {
mod bendings {
use std::{sync::Arc, time::Instant};
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockArcs, BlockBind, Inputs, RopeConf};
pub struct Bendings {
    rope_len: f64,
    winch_len: f64,
    segment: f64,
    block_arcs: BlockArcs,
    dbg: Dbg,
}
impl Bendings {
    pub fn new(parent: impl Into<String>, conf: &RopeConf, mut block_arcs: BlockArcs) -> Self {
        let dbg = Dbg::new(parent, "Bendings");
        let rope_len = conf.length.as_mm();
        log::debug!("{dbg}.new | Evaluating parking position...");
        Self {
            rope_len,
            winch_len: match block_arcs.eval() {
                Some(blocks) => {
                    let len = blocks.iter().fold(rope_len, |len, block| {
                        log::debug!("{dbg}.new | Block[{}] {:?} len {:.3} mm - wrap {:.3} mm - rope {:.3}", block.name, block.bind, len, block.wrap_length, block.rope_len_fwd);
                        match block.bind {
                            BlockBind::Drum => {
                                log::debug!("{dbg}.new | Block[{}] rope bck {:.3}", block.name, block.rope_len_bck);
                                len - block.rope_len_bck - block.wrap_length - block.rope_len_fwd
                            }
                            _ => len - block.wrap_length - block.rope_len_fwd,
                        }
                    });
                    log::debug!("{dbg}.new | Evaluating parking position - Ok, winch_len: {:.3} mm", len);
                    len
                },
                None => {
                    log::error!("{dbg}.new | Can't evaluate parking position calculations, winch_len set to default 0.0 mm");
                    0.0
                }
            },
            segment: conf.segment.as_mm(),
            block_arcs,
            dbg,
        }
    }
    pub fn eval(&mut self, inputs: &Arc<Inputs>) -> Option<Vec<Block>> {
        let t = Instant::now();
        match self.block_arcs.eval() {
            Some(blocks) => {
                match inputs.rope_pos() {
                    Some(rope_pos) => {
                        let mut start = 0.0;
                        let mut end = 0.0;
                        let mut prev_bend = start .. end;
                        let result: Vec<Block> = blocks.into_iter().filter_map(|mut block| {
                            match block.skipped {
                                true => None,
                                false => {
                                    start = match block.bind {
                                        BlockBind::Drum => self.winch_len - self.segment,
                                        BlockBind::Boom(_) => prev_bend.end,
                                        BlockBind::BoomPair(_) => prev_bend.end,
                                        BlockBind::Hook => {
                                            prev_bend.end
                                        }
                                    };
                                    end = match block.bind {
                                        BlockBind::Drum => self.winch_len + block.rope_len_bck + block.wrap_length - rope_pos,
                                        BlockBind::Hook => {
                                            start + block.wrap_length
                                        }
                                        _ => start + block.wrap_length,
                                    };
                                    prev_bend = start .. end + block.rope_len_fwd;
                                    match (end - start).abs() > 0.0 {
                                        true => {
                                            block.bending = start .. end;
                                            Some(block)
                                        }
                                        false => None,
                                    }
                                }
                            }
                        }).collect();
                        Some(result)
                    }
                    None => {
                        log::warn!("{}.eval | Rope position isn't ready", self.dbg);
                        return None;
                    }
                }
            }
            None => None,
        }
    }
}
}
mod block_arcs {
use std::f64::consts::PI;
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockBind, RopeSections};
pub struct BlockArcs {
    rope_sections: RopeSections,
    #[allow(unused)]
    dbg: Dbg,
}
impl BlockArcs {
    pub fn new(parent: impl Into<String>, rope_sections: RopeSections) -> Self {
        Self {
            rope_sections,
            dbg: Dbg::new(parent, "BlockArcs"),
        }
    }
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        match self.rope_sections.eval() {
            Some(blocks) => {
                let blocks: Vec<Block> = blocks.iter().filter_map(|block| {
                    match block.skipped {
                        true => {
                            log::debug!("{}.eval | Block {} SKIPED", self.dbg, block.name);
                            None
                        }
                        false => {
                            let wrap_alpha = match block.bind {
                                BlockBind::Drum => 0.0,
                                BlockBind::Boom(_) => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                                BlockBind::BoomPair(_) => f64::abs(block.rope_alpha_fwd - block.rope_alpha_bck),
                                BlockBind::Hook => 0.0,
                            };
                            let wrap_length = (PI * block.diameter * 0.5 * wrap_alpha) / 180.0;
                            Some(Block::new(
                                block.name.clone(),
                                block.lf,
                                block.diameter,
                                block.scheme,
                                block.bind,
                                block.rope_alpha_fwd,
                                block.rope_alpha_bck,
                                wrap_alpha,
                                wrap_length,
                                block.rope_len_fwd,
                                block.rope_len_bck,
                                0.0..0.0,
                            ))
                        }
                    }
                }).collect();
                Some(blocks)
            }
            None => None,
        }
    }
}
}
mod block {
use std::{ops::Range, str::FromStr};
use regex::Regex;
use sal_core::error::Error;
use crate::services::frdm_service::Offset;
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum BlockScheme {
    TopTop((f64, f64)) = 1,
    TopBottom((f64, f64)) = 2,
    BottomTop((f64, f64)) = 3,
    BottomBottom((f64, f64)) = 4,
}
impl BlockScheme {
    pub fn kj(&self) -> (f64, f64) {
        match self {
            BlockScheme::TopTop(kj) => *kj,
            BlockScheme::TopBottom(kj) => *kj,
            BlockScheme::BottomTop(kj) => *kj,
            BlockScheme::BottomBottom(kj) => *kj,
        }
    }
    #[allow(unused)]
    pub fn top_top() -> Self {
        Self::TopTop((-1.0, 1.0))
    }
    #[allow(unused)]
    pub fn top_bottom() -> Self {
        Self::TopBottom((1.0, 1.0))
    }
    #[allow(unused)]
    pub fn bottom_top() -> Self {
        Self::BottomTop((1.0, -1.0))
    }
    #[allow(unused)]
    pub fn bottom_bottom() -> Self {
        Self::BottomBottom((-1.0, -1.0))
    }
}
impl FromStr for BlockScheme {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TopTop" => Ok(Self::TopTop((-1.0, 1.0))),
            "TopBottom" => Ok(Self::TopBottom((1.0, 1.0))),
            "BottomTop" => Ok(Self::BottomTop((1.0, -1.0))),
            "BottomBottom" => Ok(Self::BottomBottom((-1.0, -1.0))),
            _ => Err(Error::new("BlockScheme", "from_str").err(format!("Unknown variant '{s}'"))),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockBind {
    Drum,
    Boom(usize),
    BoomPair(usize),
    Hook,
}
impl BlockBind {
    fn boom(s: &str) -> Result<Self, Error> {
        let re = Regex::new(r"(boom|boompair)[ \t](\d+)").unwrap();
        let caps = re.captures(s)
            .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0'")))?;
        let kind = caps.get(1)
            .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0 / BoomPair 0'")))?;
        let bind = caps.get(2)
            .ok_or(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0'")))?;
        let bind = bind.as_str().parse()
            .map_err(|_| Error::new("BlockBind", "from_str").err(format!("Wring Block number in '{s}', Expecting integer >= 0")))?;
        match kind.as_str() {
            "boom" => Ok(Self::Boom(bind)),
            "boompair" => Ok(Self::BoomPair(bind)),
            _ => Err(Error::new("BlockBind", "from_str").err(format!("Wrong format '{s}', Expected string like 'Boom 0 / BoomPair 0'"))),
        }
    }
    #[allow(unused)]
    pub fn is(&self, other: Self) -> bool {
        match (self, other) {
            (BlockBind::Drum, BlockBind::Drum) => true,
            (BlockBind::Boom(_), BlockBind::Boom(_)) => true,
            (BlockBind::BoomPair(_), BlockBind::BoomPair(_)) => true,
            (BlockBind::Hook, BlockBind::Hook) => true,
            _ => false,
        }
    }
}
impl FromStr for BlockBind {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase() {
            key if key == "drum" => Ok(Self::Drum),
            key if key.starts_with("boom") => Self::boom(&key),
            key if key == "hook" => Ok(Self::Hook),
            _ => Err(Error::new("BlockBind", "from_str").err(format!("Unknown variant '{s}'"))),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub name: String,
    pub lf: Offset<f64>,
    pub diameter: f64,
    pub scheme: BlockScheme,
    pub bind: BlockBind,
    pub pos: Offset<f64>,
    pub rope_alpha_fwd: f64,
    pub rope_alpha_bck: f64,
    pub wrap_alpha: f64,
    pub wrap_length: f64,
    pub rope_len_fwd: f64,
    pub rope_len_bck: f64,
    pub bending: Range<f64>,
    pub skipped: bool,
}
impl Block {
    pub fn new(
        name: impl Into<String>,
        lf: Offset<f64>,
        diameter: f64,
        scheme:BlockScheme,
        bind: BlockBind,
        rope_alpha_fwd: f64,
        rope_alpha_bck: f64,
        wrap_alpha: f64,
        wrap_length: f64,
        rope_len_fwd: f64,
        rope_len_bck: f64,
        bending: Range<f64>,
    ) -> Self {
        Self {
            name: name.into(),
            lf,
            diameter,
            scheme: scheme,
            bind: bind,
            pos: Offset::new(0.0, 0.0),
            rope_alpha_fwd,
            rope_alpha_bck,
            wrap_alpha,
            wrap_length,
            rope_len_fwd,
            rope_len_bck,
            bending,
            skipped: false,
        }
    }
}
impl Default for Block {
    fn default() -> Self {
        Self {
            name: Default::default(),
            lf: Offset::new(0.0, 0.0),
            diameter: Default::default(),
            scheme: BlockScheme::top_top(),
            bind: BlockBind::Drum,
            pos: Offset::new(0.0, 0.0),
            rope_alpha_fwd: Default::default(),
            rope_alpha_bck: Default::default(),
            wrap_alpha: Default::default(),
            wrap_length: Default::default(),
            rope_len_fwd: Default::default(),
            rope_len_bck: Default::default(),
            bending: Default::default(),
            skipped: Default::default(),
        }
    }
}
}
mod blocks {
use std::collections::VecDeque;
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Block, BlockBind, BlockConf, Boom, Booms, Offset};
pub struct Blocks {
    items: Vec<Block>,
    aux_length: f64,
    winch_rope_alpha: f64,
    parking: bool,
    booms: Booms,
    #[allow(unused)]
    dbg: Dbg,
}
impl Blocks {
    pub fn new(parent: impl Into<String>, aux_length: ConfDistance, conf: &Vec<(String, BlockConf)>, parking: bool, booms: Booms) -> Self {
        Self {
            aux_length: aux_length.as_mm(),
            winch_rope_alpha: 0.0,
            parking,
            items: conf.iter().map(|(key, conf)| Block::new(
                key,
                Offset::new(conf.lf.x.as_mm(), conf.lf.y.as_mm()),
                conf.d.as_mm(),
                conf.scheme,
                conf.bind,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0..0.0,
            )).collect(),
            booms,
            dbg: Dbg::new(parent, "Blocks"),
        }
    }
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        match self.booms.eval() {
            Some(booms) => {
                let mut blocks = VecDeque::from(self.items.clone());
                match blocks.pop_front() {
                    Some(mut block) => {
                        block.pos = self.blocks_pos(&block, &booms, &Block::default(), false);
                        let mut result = Vec::with_capacity(blocks.len());
                        let mut skipped = None;
                        let mut winch_dl;
                        while let Some(mut next) = blocks.pop_front() {
                            next.pos = self.blocks_pos(&next, &booms, &block, skipped.is_some());
                            let (k, j) = block.scheme.kj();
                            let l_block = block.pos.distance(next.pos);
                            let alpha_block = block.pos.alpha_horiz(&next.pos, l_block);
                            let rope_alpha_fwd = alpha_block + j * ((0.5 * (block.diameter + k * next.diameter) / l_block).asin().to_degrees());
                            if let BlockBind::Drum = block.bind {
                                if self.parking {
                                    self.winch_rope_alpha = rope_alpha_fwd;
                                    log::debug!("{}.eval | Block: {}: winch_rope_alpha: {:.3}", self.dbg, block.name, rope_alpha_fwd);
                                    self.parking = false;
                                }
                                winch_dl = (rope_alpha_fwd - self.winch_rope_alpha).to_radians() * block.diameter * 0.5;
                                block.rope_len_bck = winch_dl;
                            }
                            if next.bind.is(BlockBind::BoomPair(0)) && rope_alpha_fwd > 90.0 {
                                next.skipped = true;
                                skipped = Some(next);
                            } else {
                                block.rope_alpha_fwd = rope_alpha_fwd;
                                next.rope_alpha_bck = rope_alpha_fwd;
                                result.push(block);
                                if let Some(skipped) = skipped.take() {
                                    result.push(skipped);
                                }
                                block = next;
                            }
                        }
                        result.push(block);
                        Some(result)
                    }
                    None => None,
                }
            },
            None => None,
        }
    }
    fn blocks_pos(&self, block: &Block, booms: &Vec<Boom>, prev: &Block, skipped: bool) -> Offset<f64> {
        match block.bind {
            BlockBind::Drum => {
                let Offset{x: dx1, y: dy1} = rotate_xy(- block.lf.x, block.lf.y, 0.0);
                let Offset{x: dx2, y: dy2} = rotate_xy(booms[0].l4, booms[0].l3, 90.0);
                Offset::new(dx1 + dx2, dy1 + dy2)
            }
            BlockBind::Boom(index) => {
                let base_point = booms[index].gpt;
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                Offset::new(base_point.x + dx, base_point.y + dy)
            }
            BlockBind::BoomPair(index) => {
                let base_point = booms[index].gpt;
                let Offset{x: dx, y: dy} = rotate_xy(block.lf.x, block.lf.y, booms[index].alpha);
                Offset::new(base_point.x + dx, base_point.y + dy)
            }
            BlockBind::Hook => {
                Offset::new(
                    match skipped {
                        true => prev.pos.x - 0.5 * prev.diameter,
                        false => prev.pos.x + 0.5 * prev.diameter,
                    },
                    prev.pos.y - self.aux_length,
                )
            }
        }
    }
}
}
mod boom {
use crate::services::frdm_service::{InputKind, Offset};
#[derive(Debug, Clone)]
pub struct Boom {
    pub name: String,
    pub alpha_input: Option<String>,
    pub len_input: Option<String>,
    pub alpha_rel: f64,
    pub alpha: f64,
    pub len: f64,
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub l4: f64,
    pub dpt: Offset<f64>,
    pub gpt: Offset<f64>,
    pub parking: f64
}
impl Boom {
    pub fn new(name: impl Into<String>, alpha_input: InputKind<f64>, len_input: InputKind<f64>, l1: f64, l2: f64, l3: f64, l4: f64, parking: f64) -> Self {
        Self {
            name: name.into(),
            alpha_input: match &alpha_input {
                InputKind::Const(_) => None,
                InputKind::Point(val) => Some(val.clone()),
            },
            len_input: match &len_input {
                InputKind::Const(_) => None,
                InputKind::Point(val) => Some(val.clone()),
            },
            alpha_rel: match &alpha_input {
                InputKind::Const(val) => *val,
                InputKind::Point(_) => 0.0,
            },
            alpha: 0.0,
            len: match len_input {
                InputKind::Const(val) => val,
                InputKind::Point(_) => 0.0,
            },
            l1,
            l2,
            l3,
            l4,
            dpt: Offset::new(0.0, 0.0),
            gpt: Offset::new(0.0, 0.0),
            parking,
        }
    }
}
}
mod booms {
use std::sync::Arc;
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{rope_deprecation::rotate_xy, Boom, BoomConf, InputKind, Inputs, Offset};
pub struct Booms {
    items: Vec<Boom>,
    inputs: Arc<Inputs>,
    parking: bool,
    dbg: Dbg,
}
impl Booms {
    pub fn new(parent: impl Into<String>, conf: &Vec<(String, BoomConf)>, inputs: Arc<Inputs>, parking: bool) -> Self {
        let dbg = Dbg::new(parent, "Booms");
        Self {
            items: conf.iter().map(|(name, conf)| {
                let alpha = match &conf.angle {
                    InputKind::Const(angle) => InputKind::Const(*angle),
                    InputKind::Point(key) => {
                        inputs.subscribe(key);
                        InputKind::Point(key.clone())
                    }
                };
                let len = match &conf.len {
                    InputKind::Const(len) => InputKind::Const(len.as_mm()),
                    InputKind::Point(key) => {
                        inputs.subscribe(key);
                        InputKind::Point(key.clone())
                    }
                };
                Boom::new(
                    name,
                    alpha,
                    len,
                    conf.l1.as_mm(),
                    conf.l2.as_mm(),
                    conf.l3.as_mm(),
                    conf.l4.as_mm(),
                    conf.parking,
                )
            }).collect(),
            inputs,
            parking,
            dbg,
        }
    }
    pub fn eval(&mut self,) -> Option<Vec<Boom>> {
        match self.angles() {
            Some(booms) => {
                self.boom_d_g_points(booms)
            }
            None => None,
        }
    }
    fn angles(&mut self) -> Option<Vec<Boom>> {
        let mut alpha_sum = 0.0;
        let mut result = vec![];
        for (i, mut boom) in self.items.iter().cloned().enumerate() {
            let alpha_rel = match self.parking {
                true => boom.parking,
                false => match &boom.alpha_input {
                    Some(input) => match self.inputs.get(input) {
                        Some(alpha) => alpha,
                        None => {
                            log::warn!("{}.angles | Boom[{i}] '{}':  Input '{}' - Not found", self.dbg, boom.name, input);
                            return None
                        }
                    }
                    None => boom.alpha_rel,
                },
            };
            alpha_sum += alpha_rel;
            boom.alpha = alpha_sum - (i as f64) * 180.0;
            result.push(boom);
        }
        if self.parking {
            self.parking = false;
        }
        Some(result)
    }
    fn boom_d_g_points(&mut self, mut booms: Vec<Boom>) -> Option<Vec<Boom>> {
        match booms.first() {
            Some(first) => {
                let mut prev_gpt = first.gpt;
                let mut prev_alpha = first.alpha;
                for (i, boom) in booms.iter_mut().enumerate() {
                    let (x0, y0, alpha_prime) = if i == 0 {
                        (0.0, 0.0, 90.0)
                    } else {
                        (prev_gpt.x, prev_gpt.y, prev_alpha)
                    };
                    let Offset{x: wx, y: wy} = rotate_xy(boom.l4, boom.l3, alpha_prime);
                    let start = Offset::new(x0 + wx, y0 + wy);
                    let Offset{x: dx, y: dy} = rotate_xy(- boom.l2, boom.l1, boom.alpha);
                    let dpt = Offset::new(start.x + dx, start.y + dy);
                    let boom_len = match &boom.len_input {
                        Some(input) => match self.inputs.get(input) {
                            Some(len) => len,
                            None => {
                                log::warn!("{}.angles | Boom[{i}] '{}':  Input '{:?}' - Not found", self.dbg, boom.name, boom.len_input);
                                return None
                            }
                        },
                        None => boom.len,
                    };
                    let Offset{x: gx, y: gy} = rotate_xy(boom_len - boom.l2, boom.l1, boom.alpha);
                    let gpt = Offset::new(start.x + gx, start.y + gy);
                    boom.dpt = dpt;
                    boom.gpt = gpt;
                    prev_gpt = boom.gpt;
                    prev_alpha = boom.alpha;
                }
                Some(booms)
            }
            None => {
                log::warn!("{}.boom_d_g_points | No Boom's found", self.dbg);
                None
            }
        }
    }
}
}
mod input_kind {
#[derive(Debug, Clone, PartialEq)]
pub enum InputKind<T> {
    Const(T),
    Point(String),
}}
mod deprecation {
use std::{ops::Range, sync::Arc};
use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;
use crate::services::frdm_service::{Bendings, CraneConf, Inputs};
pub struct Deprecation<'a> {
    inputs: Arc<Inputs>,
    conf: CraneConf,
    segment: f64,
    ///                Block     Slices
    blocks: FxIndexMap<usize, Vec<usize>>,
    bendings: Bendings,
    results: Box<dyn Fn(&usize, f64) + 'a>,
    dbg: Dbg,
}
impl<'a> Deprecation<'a> {
    pub fn new(parent: impl Into<String>, conf: &CraneConf, inputs: Arc<Inputs>, bendings: Bendings, results: impl Fn(&usize, f64) + 'a) -> Self {
        let dbg = Dbg::new(parent, "Deprecation");
        inputs.subscribe(conf.rope.load.clone());
        inputs.subscribe(conf.rope.pos.clone());
        Self {
            inputs,
            conf: conf.clone(),
            segment: conf.rope.segment.as_mm(),
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
                            let deprecation = load / (block.diameter * 0.001);
                            let current = self.slices(&block.bending);
                            for slice in &self.blocks[block_ix] {
                                if let Err(_) = current.binary_search(slice) {
                                    (self.results)(&slice, deprecation);
                                }
                            }
                            for slice in &current {
                                if let Err(_) = self.blocks[block_ix].binary_search(slice) {
                                    (self.results)(&slice, deprecation);
                                }
                            }
                            self.blocks[block_ix] = current;
                        }
                        Some(())
                    }
                }
            },
            None => None,
        }
    }
    fn slices(&self, bend: &Range<f64>) -> Vec<usize> {
        let first_slice = (bend.start / self.segment).trunc() as usize;
        let start_point = (first_slice as f64) * self.segment;
        let delta = bend.end - start_point;
        let slices = (delta / self.segment).ceil() as usize;
        Vec::from_iter(first_slice..first_slice + slices)
    }
}
;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Offset<T> {
    pub x: T,
    pub y: T,
}
impl<T> Offset<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}
impl<T: Copy + Add<Output = T> + Mul<Output = T> + Sub<Output = T> + Sqrt<T>> Offset<T> {
    pub fn distance(&self, other: Self) -> T {
        let dif_x = other.x - self.x;
        let dif_y = other.y - self.y;
        (dif_x * dif_x + dif_y * dif_y).sqrt_()
    }
}
impl Offset<f64> {
    pub fn alpha_horiz(&self, other: &Self, length: f64) -> f64 {
        if length == 0.0 {
            return 0.0
        }
        let a =  (((self.y - other.y) / length).asin()).to_degrees();
        if self.x <= other.x {
            a
        } else {
            180.0 - a
        }
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Offset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offset({}, {})", self.x, self.y)
    }
}
impl<T: std::fmt::Display> std::fmt::Debug for Offset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offset({}, {})", self.x, self.y)
    }
}
pub trait Sqrt<T> {
    fn sqrt_(&self) -> T;
}
impl Sqrt<f64> for f64 {
    fn sqrt_(&self) -> f64 {
        f64::sqrt(*self)
    }
}
impl Sqrt<f32> for f32 {
    fn sqrt_(&self) -> f32 {
        f32::sqrt(*self)
    }
}
}
mod rope_sections {
use std::collections::VecDeque;
use sal_core::dbg::Dbg;
use crate::services::frdm_service::{Block, BlockBind, Blocks, Offset};
pub struct RopeSections {
    blocks: Blocks,
    #[allow(unused)]
    dbg: Dbg,
}
impl RopeSections {
    pub fn new(parent: impl Into<String>, blocks: Blocks) -> Self {
        Self {
            blocks,
            dbg: Dbg::new(parent, "RopeSections"),
        }
    }
    pub fn eval(&mut self) -> Option<Vec<Block>> {
        match self.blocks.eval() {
            Some(blocks) => {
                let mut blocks = VecDeque::from(blocks);
                match blocks.pop_front() {
                    Some(mut block) => {
                        if block.bind.is(BlockBind::Drum) {
                            let mut result = Vec::with_capacity(blocks.len());
                            while let Some(mut next) = blocks.pop_front() {
                                if next.skipped {
                                    continue;
                                }
                                let (k, j) = block.scheme.kj();
                                let block_x = block.pos.x + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().sin();
                                let block_y = block.pos.y + j * 0.5 * block.diameter * block.rope_alpha_fwd.to_radians().cos();
                                let next_x = next.pos.x - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().sin();
                                let next_y = next.pos.y - j * k * 0.5 * next.diameter * block.rope_alpha_fwd.to_radians().cos();
                                let rope_len_fwd = Offset::new(next_x, next_y).distance(Offset::new(block_x, block_y));
                                block.rope_len_fwd = rope_len_fwd;
                                result.push(block);
                                next.rope_len_bck = rope_len_fwd;
                                block = next;
                            }
                            result.push(block);
                            Some(result)
                        } else {
                            log::warn!("{}.eval | First block expected 'Fixed', but found {:?}", self.dbg, block.bind);
                            None
                        }
                    }
                    None => {
                        log::warn!("{}.eval | No blocks found", self.dbg);
                        None
                    }
                }
            }
            None => None,
        }
    }
}}
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
pub(super) fn rotate_xy(lx: f64, ly: f64, alpha: f64) -> Offset<f64> {
    let angle_rad = alpha.to_radians();
    Offset::new(lx * angle_rad.cos() - ly * angle_rad.sin(), lx * angle_rad.sin() + ly * angle_rad.cos())
}
}
mod block_conf {
use std::str::FromStr;
use regex::Regex;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfAngle, ConfDistance, ConfTree, ConfTreeGet}, entity::Name};
use crate::services::frdm_service::{BlockBind, BlockScheme, Offset};
#[derive(Debug, Clone, PartialEq)]
pub struct BlockConf {
    pub lf: Offset<ConfDistance>,
    pub d: ConfDistance,
    pub scheme: BlockScheme,
    pub bind: BlockBind,
    pub deflector: Option<ConfAngle>
}
impl BlockConf {
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "BlockConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let lf: String = conf.get("lf").expect(&format!("{dbg}.new | 'lf' - not found or wrong config"));
        let re = Regex::new("(.+),[ \t](.+)").unwrap();
        let lf_caps = re.captures(&lf).expect(&format!("{dbg}.new | 'lf' - not found or wrong config"));
        let lfx = ConfDistance::from_str(lf_caps.get(1).expect(&format!("{dbg}.new | 'lf.x' - wrong config")).as_str())
            .expect(&format!("{dbg}.new | 'lf.x' - wrong config"));
        let lfy = ConfDistance::from_str(lf_caps.get(2).expect(&format!("{dbg}.new | 'lf.y' - wrong config")).as_str())
            .expect(&format!("{dbg}.new | 'lf.y' - wrong config"));
        let d = conf.get_distance("d").expect(&format!("{dbg}.new | 'd' - not found or wrong config"));
        let scheme: String = conf.get("scheme").expect(&format!("{dbg}.new | 'scheme' - not found or wrong config"));
        let scheme = BlockScheme::from_str(&scheme).expect(&format!("{dbg}.new | 'scheme' - wrong config"));
        let bind: String = conf.get("bind").expect(&format!("{dbg}.new | 'bind' - not found or wrong config"));
        let bind = BlockBind::from_str(&bind).expect(&format!("{dbg}.new | 'bind' - wrong config"));
        let deflector: Option<String> = conf.get("deflector-angle");
        let deflector = deflector.map(|deflector| ConfAngle::from_str(&deflector).expect(&format!("{dbg}.new | 'deflector-angle' - wrong config")));
        Self {
            lf: Offset::new(lfx, lfy),
            d,
            scheme,
            bind,
            deflector,
        }
    }
}
}
mod boom_conf {
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name};
use crate::services::frdm_service::InputKind;
#[derive(Debug, Clone, PartialEq)]
pub struct BoomConf {
    pub l1: ConfDistance,
    pub l2: ConfDistance,
    pub l3: ConfDistance,
    pub l4: ConfDistance,
    pub len: InputKind<ConfDistance>,
    pub angle: InputKind<f64>,
    pub parking: f64,
}
impl BoomConf {
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "BoomConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let l1 = conf.get_distance("l1").expect(&format!("{dbg}.new | 'l1' - not found or wrong config"));
        let l2 = conf.get_distance("l2").expect(&format!("{dbg}.new | 'l2' - not found or wrong config"));
        let l3 = conf.get_distance("l3").expect(&format!("{dbg}.new | 'l3' - not found or wrong config"));
        let l4 = conf.get_distance("l4").expect(&format!("{dbg}.new | 'l4' - not found or wrong config"));
        let len = match conf.get_distance("len") {
            Ok(len) => InputKind::Const(len),
            Err(_) => InputKind::Point(conf.get_fn_config(&dbg, "len", &mut vec![])
                .expect(&format!("{dbg}.new | 'len' - can be Const: 11200.0 mm or point real 'App/MultiQueue/Load.MainBoomAngle', but found '{:?}'", ConfTreeGet::<String>::get(&conf, "len")))
                .name()),
        };
        let angle = match conf.get("angle") {
            Some(angle) => InputKind::Const(angle),
            None => InputKind::Point(conf.get_fn_config(&dbg, "angle", &mut vec![])
                .expect(&format!("{dbg}.new | 'angle' - can be Const: 11200.0 mm or point real 'App/MultiQueue/Load.MainBoomAngle', but found '{:?}'", ConfTreeGet::<String>::get(&conf, "len")))
                .name()),
        };
        let parking = conf.get("parking")
            .expect(&format!("{dbg}.new | 'parking' - missed or wrong config", ));
        Self {
            l1,
            l2,
            l3,
            l4,
            len,
            angle,
            parking,
        }
    }
}
}
mod crane_conf {
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use crate::services::frdm_service::{BlockConf, BoomConf, RopeConf};
#[derive(Debug, Clone, PartialEq)]
pub struct CraneConf {
    pub booms: Vec<(String, BoomConf)>,
    pub blocks: Vec<(String, BlockConf)>,
    pub rope: RopeConf,
}
impl CraneConf {
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "CraneConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let booms: &Vec<serde_yaml::Value> = conf.get("booms").expect(&format!("{dbg}.new | 'booms' - not found or wrong config"));
        let booms = booms.iter().map(|boom| {
            let (key, boom) = boom.as_mapping()
                .expect(&format!("{dbg}.new | boom's config have to be a Map, but found: {:#?}", boom))
                .iter()
                .next()
                .expect(&format!("{dbg}.new | 'boom' config can't be empty, but found: {:#?}", boom));
            let boom = ConfTree::new(key.as_str().unwrap(), boom.to_owned());
            (boom.key.clone(), BoomConf::new(&name, boom))
        }).collect();
        log::trace!("{dbg}.new | booms: {:#?}", booms);
        let blocks: &Vec<serde_yaml::Value> = conf.get("blocks").expect(&format!("{dbg}.new | 'blocks' - not found or wrong config"));
        let blocks = blocks.iter().map(|block| {
            let (key, block) = block.as_mapping()
                .expect(&format!("{dbg}.new | block's config have to be a Map, but found: {:#?}", block))
                .iter()
                .next()
                .expect(&format!("{dbg}.new | 'block' config can't be empty, but found: {:#?}", block));
            let key = if key.is_number() {
                format!("{}", key.as_u64().expect(&format!("{dbg}.new | Block's key expected positive number or string")))
            } else if key.is_string() {
                format!("{}", key.as_str().expect(&format!("{dbg}.new | Block's key expected positive number or string")))
            } else {
                panic!("{dbg}.new | Block's key expected positive number or string");
            };
            let block = ConfTree::new(key, block.to_owned());
            (block.key.clone(), BlockConf::new(&name, block))
        }).collect();
        log::trace!("{dbg}.new | blocks: {:#?}", blocks);
        let rope = conf.get("rope").expect(&format!("{dbg}.new | 'rope' - not found or wrong config"));
        let rope = RopeConf::new(&name, rope);
        log::trace!("{dbg}.new | rope: {:#?}", rope);
        Self {
            booms,
            blocks,
            rope,
        }
    }
}
impl Default for CraneConf {
    fn default() -> Self {
        Self {
            booms: Default::default(),
            blocks: Default::default(),
            rope: Default::default(),
        }
    }
}}
mod rope_conf {
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree}, entity::Name};
#[derive(Debug, Clone, PartialEq)]
pub struct RopeConf {
    pub width: ConfDistance,
    pub length: ConfDistance,
    pub aux_length: ConfDistance,
    pub segment: ConfDistance,
    pub pos: String,
    pub load: String,
}
impl RopeConf {
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "RopeConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let width = conf.get_distance("width").expect(&format!("{dbg}.new | 'width' - not found or wrong configuration"));
        log::trace!("{dbg}.new | width: {:?}", width);
        let length = conf.get_distance("length").expect(&format!("{dbg}.new | 'length' - not found or wrong configuration"));
        log::trace!("{dbg}.new | length: {:?}", length);
        let aux_length = conf.get_distance("aux-length").expect(&format!("{dbg}.new | 'aux-length' - not found or wrong configuration"));
        log::trace!("{dbg}.new | aux-length: {:?}", aux_length);
        let segment = conf.get_distance("segment").expect(&format!("{dbg}.new | 'segment' - not found or wrong configuration"));
        log::trace!("{dbg}.new | segment: {:?}", segment);
        let pos = conf.get_fn_config(&dbg, "pos", &mut vec![]).unwrap().name();
        log::trace!("{dbg}.new | pos: {:?}", pos);
        let load = conf.get_fn_config(&dbg, "load", &mut vec![]).unwrap().name();
        log::trace!("{dbg}.new | load: {:?}", load);
        Self {
            width,
            length,
            aux_length,
            segment,
            pos,
            load,
        }
    }
}
impl Default for RopeConf {
    fn default() -> Self {
        Self {
            width: Default::default(),
            length: Default::default(),
            aux_length: Default::default(),
            segment: Default::default(),
            pos: Default::default(),
            load: Default::default(),
        }
    }
}}
mod rope_deprecation_conf {
use std::time::Duration;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use crate::{infra::ApiClientConf, services::frdm_service::CraneConf};
#[derive(Debug, PartialEq, Clone)]
pub struct RopeDeprecationConf {
    pub name: Name,
    pub wait_started: Option<Duration>,
    pub api: ApiClientConf,
    pub table: String,
    pub crane: CraneConf,
}
impl RopeDeprecationConf {
    pub fn new(parent: impl Into<String>, conf: ConfTree, api: ApiClientConf) -> Self {
        let parent = parent.into();
        let me = "RopeDeprecationConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let table = conf.get("table").expect(&format!("{dbg}.new | 'table' - not found or wrong config"));
        log::trace!("{dbg}.new | table: {:?}", table);
        let crane = conf.get("crane").expect(&format!("{dbg}.new | 'crane' - not found or wrong config"));
        let crane = CraneConf::new(&name, crane);
        log::trace!("{dbg}.new | crane: {:?}", crane);
        Self {
            name,
            wait_started,
            api,
            table,
            crane,
        }
    }
}
impl Default for RopeDeprecationConf {
    fn default() -> Self {
        Self {
            name: Name::new("", "RopeDeprecationConf"),
            wait_started: Default::default(),
            api: Default::default(),
            table: Default::default(),
            crane: Default::default(),
        }
    }
}}
mod rope_deprecation {
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Name, Object}, Service, ServiceWaiting, RECV_TIMEOUT},
    sync::{channel::RecvTimeoutError, Handles},
    thread_pool::Scheduler,
};
use crate::{infra::ApiClient, services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, Deprecation, Inputs, RopeSections, RopeDeprecationConf}};
pub struct RopeDeprecation {
    name: Name,
    conf: RopeDeprecationConf,
    inputs: Arc<Inputs>,
    api_client: Arc<ApiClient>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
impl RopeDeprecation {
    pub fn new(
        parent: impl Into<String>,
        conf: RopeDeprecationConf,
        inputs: Arc<Inputs>,
        api_client: Arc<ApiClient>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDeprecation");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            inputs,
            api_client,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
}
impl Object for RopeDeprecation {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
impl std::fmt::Debug for RopeDeprecation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RopeDeprecation")
            .field("dbg", &self.dbg)
            .finish()
    }
}
// ///
// /// Used for logging
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// enum NotifyState {
//     Start,
//     Exit,
//     SendError,
// }
//
//
impl Service for RopeDeprecation where {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let inputs = self.inputs.clone();
        let exit = self.exit.clone();
        let api_client = self.api_client.clone();
        let mut handles = vec![];
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let recv = inputs.listen();
            let conf_table = conf.table.clone();
            let mut deprecation = Deprecation::new(
                dbg,
                &conf.crane,
                inputs.clone(),
                Bendings::new(
                    dbg,
                    &conf.crane.rope,
                    BlockArcs::new(
                        dbg,
                        RopeSections::new(
                            dbg,
                            Blocks::new(
                                dbg,
                                conf.crane.rope.aux_length,
                                &conf.crane.blocks,
                                true,
                                Booms::new(dbg, &conf.crane.booms, inputs, true),
                            ),
                        ),
                    ),
                ),
                |slice_ix, deprecation| {
                    log::debug!("{dbg}.run | Deprecation at slice {slice_ix}: {:?}", deprecation);
                    let sql = format!(r"
                        insert into {conf_table} (id, deprecation) values ({slice_ix}, {deprecation})
                        on conflict (id) do update
                            set deprecation = {conf_table}.deprecation + {deprecation} where {conf_table}.id = {slice_ix};
                    ");
                    // log::trace!("{dbg}.run | Fetching sql: {:?}", sql);
                    api_client.fetch(sql).then(
                        |_reply| {
                            // log::trace!("{dbg}.run | Sql reply: {:?}", reply);
                        },
                        |err| {
                            log::warn!("{dbg}.run | Sql error: {:?}", err);
                        },
                    );
                },
            );
            service_release.add(Ok(()));
            while !exit.load(Ordering::Acquire) {
                log::trace!("{dbg}.run | Receiving events...");
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        log::debug!("{dbg}.run | Received event: {}: {}", point.name(), point.to_string().as_string().value);
                        deprecation.eval();
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {}
                        _ => {
                            log::warn!("{dbg}.run | Recv error: {:?}", err);
                            break;
                        }
                    },
                }
            }
            log::info!("{dbg}.run | Exit");
        });
        handles.push(handle);
        for handle in handles {
            match handle {
                Ok(handle) => {
                    self.handles.push(handle);
                }
                Err(err) => {
                    let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                    log::warn!("{}", err);
                    return Err(err);
                }
            }
        }
        let r = match conf.wait_started {
            Some(_) => {
                log::info!("{}.run | Waiting while starting...", self.dbg);
                service_waiting.wait()
            }
            None => Ok(()),
        };
        log::info!("{}.run | Starting - ok", self.dbg);
        r
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}
}
pub(crate) use algorithm::*;
pub(crate) use block_conf::*;
pub(crate) use boom_conf::*;
pub(crate) use crane_conf::*;
pub(crate) use rope_conf::*;
pub(crate) use rope_deprecation_conf::*;
pub(crate) use rope_deprecation::*;
}
mod frdm_service_conf {
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use std::{fs, time::Duration};
use crate::{infra::ApiClientConf, services::frdm_service::{rope_defect::RopeDefectConf, rope_deprecation::RopeDeprecationConf}};
///
/// Config for FrdmService format:
/// ```yaml
/// service FrdmService FrdmService1:
///     cycle: 100 ms
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     subscribe: MultiQueue       # Service name, to subscribe for event's required for the calculations like rope positin and crane angles
#[derive(Debug, Clone, PartialEq)]
pub struct FrdmServiceConf {
    pub name: Name,
    pub wait_started: Option<Duration>,
    pub subscribe: String,
    pub api: ApiClientConf,
    pub table_settings: String,
    pub rope_defect: RopeDefectConf,
    pub rope_deprecation: RopeDeprecationConf,
}
impl FrdmServiceConf {
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("FrdmServiceConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong config"));
        log::trace!("{dbg}.new | subscribe: {:?}", subscribe);
        let api: ConfTree = conf.get("api-client").expect(&format!("{dbg}.new | 'api-client' - not found or wrong config"));
        let api = ApiClientConf::new(&name, api);
        log::trace!("{dbg}.new | api: {:#?}", api);
        let table_settings = conf.get("table-settings").expect(&format!("{dbg}.new | 'table-settings' - not found or wrong config"));
        log::trace!("{dbg}.new | table-settings: {:?}", table_settings);
        let rope_defect: ConfTree = conf.get("rope-defect").expect(&format!("{dbg}.new | 'rope-defect' - not found or wrong config"));
        let rope_defect = RopeDefectConf::new(&name, rope_defect, api.clone());
        log::trace!("{dbg}.new | rope-defect: {:#?}", rope_defect);
        let rope_deprecation: ConfTree = conf.get("rope-deprecation").expect(&format!("{dbg}.new | 'rope-deprecation' - not found or wrong config"));
        let rope_deprecation = RopeDeprecationConf::new(&name, rope_deprecation, api.clone());
        log::trace!("{dbg}.new | rope-deprecation: {:#?}", rope_deprecation);
        Self {
            name,
            wait_started,
            subscribe,
            api,
            table_settings,
            rope_defect,
            rope_deprecation,
        }
    }
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> FrdmServiceConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("FrdmServiceConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> FrdmServiceConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        FrdmServiceConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("FrdmServiceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("FrdmServiceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
impl Default for FrdmServiceConf {
    fn default() -> Self {
        Self {
            name: Name::new("", "FrdmServiceConf"),
            wait_started: Default::default(),
            subscribe: Default::default(),
            api: Default::default(),
            table_settings: Default::default(),
            rope_defect: Default::default(),
            rope_deprecation: Default::default(),
        }
    }
}
}
mod frdm_service {
use std::{path::Path, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use dashmap::DashMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Name, Object}, Service, Services},
    thread_pool::Scheduler,
};
use crate::{infra::ApiClient, services::frdm_service::{FrdmServiceConf, Inputs, RopeDefect, RopeDeprecation}};
pub struct FrdmService {
    name: Name,
    conf: FrdmServiceConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    tasks: Arc<DashMap<String, Arc<dyn Service>>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
impl FrdmService {
    pub fn new(conf: FrdmServiceConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            tasks: Arc::new(DashMap::new()),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    fn update_db_settings(&self, winch: usize, api_client: Arc<ApiClient>, exit: Arc<AtomicBool>) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let table = self.conf.table_settings.clone();
        let rope_length = self.conf.rope_deprecation.crane.rope.length.as_m();
        let defect_slices = (rope_length / self.conf.rope_defect.segment.as_m()).round() as usize;
        let deprecation_slices = (rope_length / self.conf.rope_deprecation.crane.rope.segment.as_m()).round() as usize;
        let _ = self.scheduler.spawn(move || {
            log::debug!("{dbg}.update_db_settings | Updating db settings...");
            let sql = format!(r"
                do $$
                begin
                    insert into {table} (id, value) values ('winch{winch}-rope_length', {rope_length})
                    on conflict (id) do
                        update set value = {rope_length} where {table}.id = 'winch{winch}-rope_length';
                    insert into {table} (id, value) values ('winch{winch}-defect_slices', {defect_slices})
                    on conflict (id) do
                        update set value = {defect_slices} where {table}.id = 'winch{winch}-defect_slices';
                    insert into {table} (id, value) values ('winch{winch}-deprecation_slices', {deprecation_slices})
                    on conflict (id) do
                        update set value = {deprecation_slices} where {table}.id = 'winch{winch}-deprecation_slices';
                end; $$
                language plpgsql;
            ");
            log::trace!("{dbg}.update_db_settings | Fetching sql: {:?}", sql);
            while !exit.load(Ordering::Acquire) {
                match api_client.fetch(&sql).wait() {
                    Ok(reply) => {
                        if reply.is_ok() {
                            log::debug!("{dbg}.update_db_settings | Updating db settings - Ok {:?}", reply.unwrap());
                            break;
                        }
                        log::warn!("{dbg}.update_db_settings | Sql reply: {:?}", reply);
                    },
                    Err(err) => {
                        log::error!("{dbg}.update_db_settings | Fetch error: {:?}", err);
                    }
                }
            }
        })?;
        Ok(())
    }
    fn create_rope_defects_dir(&self, path: &Path) -> Result<(), Error> {
        std::fs::create_dir_all(path).map_err(|err| Error::new(&self.dbg, "create_rope_defects_dir").pass(err.to_string()))
    }
}
impl Object for FrdmService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
impl std::fmt::Debug for FrdmService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrdmService")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
//
impl Service for FrdmService {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let name = self.name.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let storage_path = Path::new("assets/files").join(
            self.name.join()
                .chars()
                .enumerate()
                .filter(|(ix, ch)| !((*ix == 0) & (*ch == '/')))
                .map(|(_, ch)| ch)
                .collect::<String>()
        );
        if let Err(err) = self.create_rope_defects_dir(&storage_path) {
            log::warn!("{}.run | Can't create folder for rope defects images: {:?}", self.dbg, err);
        }
        let api_client = Arc::new(ApiClient::new(conf.api.clone(), scheduler.clone()));
        self.tasks.insert(api_client.name().join(), api_client.clone());
        api_client.run()?;
        log::info!("{}.run | ApiClient ready", self.dbg);
        self.update_db_settings(1, api_client.clone(), self.exit.clone())?;
        // let subscription: Vec<SubscriptionCriteria> = [
        //         conf.rope_deprecation.crane.rope.pos.clone(),
        //         conf.rope_deprecation.crane.rope.load.clone(),
        //     ]
        //     .iter().chain(
        //         conf.rope_deprecation.crane.booms.iter().filter_map(|(_, b)| {
        //             match &b.angle {
        //                 crate::services::frdm_service::InputKind::Const(_) => None,
        //                 crate::services::frdm_service::InputKind::Point(v) => Some(v),
        //             }
        //         }),
        //     )
        //     .map(|point| {
        //         let subscription = SubscriptionCriteria::new(point, Cot::Inf);
        //         log::trace!("{dbg}.run | Subscription: {:?}", subscription);
        //         subscription
        //     })
        //     .collect();
        // let (_, recv) = services.subscribe(&conf.subscribe, &name.join(), &subscription);
        let inputs = Arc::new(Inputs::new(&name, &conf, services.clone(), scheduler.clone(), self.exit.clone()));
        self.tasks.insert(inputs.name().join(), inputs.clone());
        let rope_deprecation = Arc::new(RopeDeprecation::new(
            &self.name,
            conf.rope_deprecation,
            inputs.clone(),
            api_client.clone(),
            scheduler.clone(),
        ));
        self.tasks.insert(rope_deprecation.name().join(), rope_deprecation.clone());
        rope_deprecation.run()?;
        log::info!("{}.run | RopeDeprecation ready", self.dbg);
        if !conf.rope_defect.cameras.is_empty() {
            log::info!("{}.run | Camera's configured: {}", self.dbg, conf.rope_defect.cameras.len());
            for (camera_id, camera_conf) in &conf.rope_defect.cameras {
                log::info!("{}.run | Camera '{}' [{}]", self.dbg, camera_conf.name, **camera_id);
                let defect_detection = Arc::new(RopeDefect::new(
                    &self.name,
                    conf.rope_defect.clone(),
                    **camera_id,
                    storage_path.clone(),
                    inputs.clone(),
                    api_client.clone(),
                    scheduler.clone(),
                ));
                self.tasks.insert(defect_detection.name().join(), defect_detection.clone());
                defect_detection.run()?;
            }
        } else {
            log::warn!("{}.run | No Camera's configured", self.dbg);
        }
        inputs.run()?;      // have to be started after all subscription being added, then it will subscribe all them on MultiQueue
        log::info!("{}.run | RopeDefect's ready", self.dbg);
        log::info!("{}.run | Starting - Ok", self.dbg);
        Ok(())
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        let mut errors = vec![];
        for task in self.tasks.iter() {
            if let Err(err) = task.value().wait() {
                errors.push(err);
            }
        }
        if errors.is_empty() {
            log::info!("{}.run | Exit", self.dbg);
            Ok(())
        } else {
            Err(Error::new(&self.dbg, "wait").pass(errors.iter().fold(String::new(), |acc, err| format!("{}\n{}", acc, err))))
        }
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.tasks.iter().all(|task| task.value().is_finished())
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
        for task in self.tasks.iter() {
            task.value().exit();
        }
    }
}
}
mod inputs {
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{Service, ServiceWaiting, Services, SubscriptionCriteria, conf::ServicesConf, entity::{Cot, Name, Object, Point}, RegistryConf}, sync::{AtomicUsizeOption, Handles}, thread_pool::{Scheduler, ThreadPool}};
use crate::{domain::{RECV_TIMEOUT, unbounded, FxDashMap, Receiver, Sender}, services::frdm_service::{FrdmServiceConf, Rope}};
///
/// Stores all last incoming events, specified in the `subscriptions`
pub struct Inputs {
    name: Name,
    inputs: Arc<FxDashMap<String, Option<f64>>>,
    listeners: Arc<FxDashMap<String, Sender<Point>>>,
    conf: FrdmServiceConf,
    rope: Arc<Rope>,
    rope_pos: Arc<AtomicUsizeOption>,
    cam_segment_ix: Arc<AtomicUsizeOption>,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl Inputs {
    ///
    /// Returns [Inputs] new instance
    pub fn new(
        parent: impl Into<String>,
        conf: &FrdmServiceConf,
        services: Arc<Services>,
        scheduler: Scheduler,
        exit: Arc<AtomicBool>,
    ) -> Self {
        let name = Name::new(parent, "Inputs");
        let dbg = Dbg::new(name.parent(), name.me());
        let rope = Arc::new(Rope::new(&name, conf.rope_defect.camera_offset, conf.rope_defect.segment, conf.rope_defect.segment_threshold));
        Self {
            name,
            inputs: Arc::new(FxDashMap::default()),
            listeners: Arc::new(FxDashMap::default()),
            conf: conf.clone(),
            rope,
            rope_pos: Arc::new(AtomicUsizeOption::new(None)),
            cam_segment_ix: Arc::new(AtomicUsizeOption::new(None)),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
    ///
    /// Returns fake [Inputs] new instance for testing purposes
    /// - `data` - a array with pairs key - value, contains in the keys - names of required events, in values - corresponding values
    #[allow(unused)]
    pub(crate) fn fake(
        parent: impl Into<String>,
        conf: &FrdmServiceConf,
        data: impl IntoIterator<Item = (impl Into<String>, f64)>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        let name = Name::new(parent, "Inputs");
        let dbg = Dbg::new(name.parent(), name.me());
        let rope = Arc::new(Rope::new(&name, conf.rope_defect.camera_offset, conf.rope_defect.segment, conf.rope_defect.segment_threshold));
        let tp = ThreadPool::new(&dbg, Some(4));
        let inputs = Arc::new(FxDashMap::default());
        for (key, val) in data {
            _ = inputs.insert(key.into(), val);
        }
        Self {
            name: name.clone(),
            inputs: Arc::new(FxDashMap::default()),
            listeners: Arc::new(FxDashMap::default()),
            conf: conf.clone(),
            rope,
            rope_pos: Arc::new(AtomicUsizeOption::new(None)),
            cam_segment_ix: Arc::new(AtomicUsizeOption::new(None)),
            services: Arc::new(Services::new(
                &dbg,
                ServicesConf { name, retain: RegistryConf { path: None, point: None } },
                None,
            ).unwrap()),
            scheduler: tp.scheduler(),
            handles: Handles::new(&dbg),
            exit,
            dbg,
        }
    }
    ///
    /// Adds new value into the current state
    ///
    /// Used for internal or testing purposes only
    ///
    /// In nornal operation events should be received by the subscription
    ///
    /// Returns channel with all internal events
    pub fn listen(&self) -> Receiver<Point> {
        let key = format!("listener-{}", self.listeners.len());
        let (send, recv) = unbounded();
        _ = self.listeners.insert(key, send);
        recv
    }
    ///
    /// Add a subscription, as a `key` of the event, which later can be requested via `get(key)`
    pub fn subscribe(&self, key: impl Into<String>) {
        self.inputs.insert(key.into(), None);
    }
    ///
    /// Returns current value from inputs by the key if exists
    pub fn get(&self, key: &str) -> Option<f64> {
        match self.inputs.get(key) {
            Some(input) => *input.value(),
            None => {
                log::warn!("{}.get | Unexpected Event '{}' requested", self.dbg, key);
                None
            }
        }
    }
    ///
    /// Rope position, mm
    ///
    /// Rope position is set to zero when crane in the parking position
    ///
    /// Rope position increments as it's unwound from the winch
    pub fn rope_pos(&self) -> Option<f64> {
        match self.inputs.get(&self.conf.rope_deprecation.crane.rope.pos) {
            Some(entry) => entry.value().map(|v| v * 1000.0),
            None => None,
        }
    }
    #[allow(unused)]
    pub fn pos_at_cam(&self) -> Option<f64> {
        self.rope_pos.load().map(|pos| self.rope.pos_at_cam(pos) as f64)
    }
    pub fn cam_segment_ix(&self) -> Arc<AtomicUsizeOption> {
        self.cam_segment_ix.clone()
    }
}
impl Object for Inputs {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
impl std::fmt::Debug for Inputs {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Inputs")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
//
impl Service for Inputs where {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let inputs = self.inputs.clone();
        let listeners = self.listeners.clone();
        let rope = self.rope.clone();
        let rope_pos = self.rope_pos.clone();
        let cam_segment_ix = self.cam_segment_ix.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let points: Vec<SubscriptionCriteria> = self.inputs.iter().map(|point| {
            let subscription = SubscriptionCriteria::new(point.key(), Cot::Inf);
            log::trace!("{dbg}.run | Subscription: {:?}", subscription);
            subscription
        }).collect();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let (_, recv) = services.subscribe(&conf.subscribe, &name.join(), &points);
            service_release.add(Ok(()));
            while !exit.load(Ordering::Acquire) {
                log::trace!("{dbg}.run | Receiving points...");
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(event) => {
                        let name = event.name();
                        match inputs.get_mut(&name) {
                            Some(mut input) => {
                                log::debug!("{dbg}.run | Event '{}', value: {:?}", name, event.value());
                                if name == conf.rope_deprecation.crane.rope.pos {
                                    let pos = (event.to_double().as_double().value * 1000.0).round() as usize;
                                    rope_pos.store(Some(pos));
                                    cam_segment_ix.store(rope.segment_index(pos));
                                }
                                match &event {
                                    Point::Bool(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'Bool'", name),
                                    Point::Int(point) => _ = input.replace(point.value as f64),
                                    Point::Real(point) => _ = input.replace(point.value as f64),
                                    Point::Double(point) => _ = input.replace(point.value),
                                    Point::String(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'String'", name),
                                    Point::Bytes(_) => log::warn!("{dbg}.run | Event '{}' - expected numeric type, but has 'Bytes'", name),
                                }
                                for send in listeners.iter() {
                                    if let Err(err) = send.value().send(event.clone()) {
                                        log::warn!("{dbg}.run | Send error {:?}", err);
                                    }
                                }
                            }
                            None => log::warn!("{dbg}.run | Unexpected Event '{}'", name),
                        }
                        // Self::add_(dbg, &inputs, &listeners, &event);
                    }
                    Err(err) => match err {
                        crate::domain::RecvTimeoutError::Timeout => {}
                        _ => {
                            log::warn!("{dbg}.run | Receive error: {:?}", err);
                            break;
                        }
                    },
                }
            }
            log::info!("{dbg}.run | Exit");
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                return Err(err);
            }
        }
        let r = match conf.wait_started {
            Some(_) => {
                log::info!("{}.run | Waiting while starting...", self.dbg);
                service_waiting.wait()
            }
            None => Ok(()),
        };
        log::info!("{}.run | Starting - ok", self.dbg);
        r
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }
}
}
pub(crate) use rope_defect::*;
pub(crate) use rope_deprecation::*;
pub use frdm_service_conf::*;
pub use frdm_service::*;
pub use inputs::*;
