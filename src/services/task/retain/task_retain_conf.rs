use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};

use crate::{err, err_pass};
///
/// ### RetainMode
/// 
/// - `Debug` - formatted json useful for debugging 
/// - `Release` - fast and compact bytes
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum RetainMode {
    #[serde(alias = "debug")]
    Debug,
    #[serde(alias = "release")]
    Release,
}
///
/// ### Config for the two-stage journal writing strategy.
#[derive(Debug, Clone)]
pub struct JournalConf {
    /// Settings for the continuous flushing of the append-log.
    pub flush: FlushConf,
    /// Threshold for the second stage: triggers full journal compaction (rewriting 
    /// the state to a clean file via atomic replacement) when the append-log 
    /// reaches this size.
    /// 
    /// Recomended: `32 ... 128 MB`.
    pub compaction_limit_mb: usize,
}
///
/// ### Config for the append-log buffer flushing criteria.
/// The flush is triggered by whichever limit is reached first.
#[derive(Debug, Clone)]
pub struct FlushConf {
    /// Maximum size of unwritten data in the IO buffer before forcing a write to disk.
    /// 
    /// Recommended : `4 096 ... 65 536 bytes` (`4 KB .. 64 KB`).
    pub bytes_limit: usize,
    /// Maximum time to wait since the last flush before forcing data to disk, 
    /// 
    /// Recommended: `10 ... 30 sec`.
    pub interval: Duration,
}
///
/// ### Config | `TaskRetainConf`
/// 
/// ```yaml
/// service Task HistoryTask:
///     wait-started: 100 ms         # optional, next service will wait until current completely started plus specified time
///     cycle: 1 s
///     retain:
///         mode: release            # release - fast and compact / debug - formatted json useful for debugging 
///     in queue recv-queue:
///         max-length: 10000
///     subscribe:
///         /App/MultiQueue:                     # - multicast subscription to the MultiQueue
///             {cot: Inf, history: rw}: []               #   - on all points having Cot::Inf and history::ReadWrite
///     # fn Debug:
///     #     input: point any every
///                         ...
#[derive(Debug, Clone)]
pub struct TaskRetainConf {
    pub name: Name,
    /// Configuration for the two-stage journal writing strategy (append, compactation).
    pub journal: JournalConf,
    pub mode: RetainMode,
    dbg: Dbg,
}
//
// 
impl TaskRetainConf {
    ///
    /// Returns `TaskRetainConf` new instance
    #[named]
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Result<TaskRetainConf, Error> {
        let me = "TaskRetain";
        let parent = parent.into();
        let name = Name::new(&parent, me);
        let dbg = Dbg::new(parent, "TaskRetainConf");
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let mode = conf.get("mode").map(|v: serde_yaml::Value| serde_yaml::from_value(v)).unwrap_or(Ok(RetainMode::Release))
            .map_err(|err| err_pass!(dbg, err, "'mode' - wrong config, 'release' / 'debug' expected"))?;
        log::trace!("{}.new | mode: {:#?}", dbg, mode);
        Ok(TaskRetainConf {
            name,
            journal: JournalConf {
                flush: FlushConf {
                    bytes_limit: 16 * 1024,
                    interval: Duration::from_secs(16),
                },
                compaction_limit_mb: 32 * 1024 * 1024,
            },
            mode,
            dbg,
        })
    }
    ///
    /// Creates config from serde_yaml::Value of following format:
    #[named]
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> Result<TaskRetainConf, Error> {
        let (key, value) = value.as_mapping().unwrap().into_iter().next()
            .ok_or_else(|| err!(Self, "Wrong or empty conf: {:#?}", value))?;
        let key = key.as_str().ok_or_else(|| err!(Self, "Wrong conf: {:#?}", value))?;
        Self::new(parent, ConfTree::new(key, value.clone()))
    }
    ///
    /// Reads config from path
    #[allow(unused)]
    #[named]
    pub fn read(parent: impl Into<String>, path: &str) -> Result<TaskRetainConf, Error> {
        let yaml_string = fs::read_to_string(path)
            .map_err(|err| err_pass!(Self, err, "Can't read file '{}'", path))?;
        let conf = serde_yaml::from_str(&yaml_string)
            .map_err(|err| err_pass!(Self, err, "Can't parse conf '{:?}'", yaml_string))?;
        TaskRetainConf::from_yaml(parent, &conf)
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
        vec![]
    }
}
