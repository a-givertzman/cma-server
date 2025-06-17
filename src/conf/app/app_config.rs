use indexmap::IndexMap;
use sal_sync::services::{conf::{ConfKeywd, ConfKind, ConfTree, ConfTreeGet, ServicesConf}, entity::Name};
use std::{fs, path::Path, str::FromStr};
///
/// Creates application config from serde_yaml::Value of following format:
/// 
/// Example
/// 
/// ```yaml
/// name: ApplicationName
/// description: Short explanation / purpose etc.
/// services:
///     retain:
///         api:
///             table:      public.tags
///             address:    0.0.0.0:8080
///             auth_token: 123!@#
///             database:   cma_data_server
/// 
/// service ProfinetClient Ied01:          # device will be executed in the independent thread, must have unique name
///    in queue in-queue:
///        max-length: 10000
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                     # operating cycle time of the device
///    protocol: 'profinet'
///    description: 'S7-IED-01.01'
///    ip: '192.168.100.243'
///    rack: 0
///    slot: 1
///    db db899:                       # multiple DB blocks are allowed, must have unique namewithing parent device
///        description: 'db899 | Exhibit - drive data'
///        number: 899
///        offset: 0
///        size: 34
///        point Drive.Speed: 
///            type: 'Real'
///            offset: 0
///                 ...
/// service Task task1:
///     cycle: 1 ms
///     in queue recv-queue:
///         max-length: 10000
///     let var0: 
///         input: const real 2.224
///     
///     fn ToMultiQueue:
///         in1 point CraneMovement.BoomUp: 
///             type: 'Int'
///             comment: 'Some indication'
///             input fn Add:
///                 input1 fn Add:
///                     input1: const real 0.2
///                     input2: point real '/path/Point.Name'
///     ...
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct AppConfig {
    pub(crate) name: Name,
    pub(crate) description: String,
    // pub(crate) cycle: Option<Duration>,
    pub(crate) tread_pool: Option<usize>,
    pub(crate) nodes: IndexMap<ConfKeywd, ConfTree>,
    pub(crate) services: ServicesConf,
}
//
// 
impl AppConfig {
    ///
    /// Returns [AppConfig] new instance:
    pub fn new(conf: ConfTree) -> Self {
        log::trace!("AppConfig.new | conf: {:?}", conf);
        let name: String = conf.get("name").unwrap();
        let self_name = Name::new("", name);
        let self_id = format!("AppConfig({})", self_name);
        log::debug!("{}.new | name: {:?}", self_id, self_name);
        let description = conf.get("description").unwrap();
        log::debug!("{}.new | description: {:?}", self_id, description);
        let tread_pool = conf.get("tread_pool").map(|v: u64| v as usize);
        log::debug!("{}.new | tread_pool: {:?}", self_id, tread_pool);
        let mut nodes = IndexMap::new();
        for key in conf.keys(&["name", "description", "services", "retain"]) {
            let keyword = ConfKeywd::from_str(&key).unwrap();
            match keyword.kind() {
                k if k == ConfKind::Service.to_string() || k == ConfKind::Task.to_string() => {
                    let node_name = keyword.name();
                    let node_conf = conf.get(key).unwrap();
                    if log::max_level() == log::LevelFilter::Debug {
                        let sufix = match keyword.sufix().is_empty() {
                            true => "".to_owned(),
                            false => format!(": '{}'", keyword.sufix()),
                        };
                        log::debug!("{}.new | service '{}'{}", self_id, node_name, sufix);
                    } else if log::max_level() == log::LevelFilter::Trace {
                        log::trace!("{}.new | DB '{}'   |   conf: {:?}", self_id, node_name, node_conf);
                    }
                    nodes.insert(
                        keyword,
                        node_conf,
                    );
                }
                _ => {
                    panic!("{}.new | Node '{:?}' - is not allowed in the root of the application config", self_id, keyword);
                }
            }
        }
        let services = conf.get("services").unwrap();
        let services = ServicesConf::new(&self_id, services);
        log::debug!("{}.new | services: {:#?}", self_id, services);
        // let services = RetainConf::default();
        Self {
            name: self_name,
            description,
            tread_pool,
            nodes,
            services,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml_value(value: &serde_yaml::Value) -> AppConfig {
        Self::new(ConfTree::new_root(value.clone()))
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read<P>(path: Vec<P>) -> AppConfig where P: AsRef<Path> {
        let self_id = "AppConfig";
        log::info!("{}.read | Reading configuration files...", self_id);
        let mut files = vec![];
        for p in path {
            match fs::read_to_string(&p) {
                Ok(f) => {
                    files.push(f)
                }
                Err(err) => {
                    panic!("{}.read | File '{}' reading error: {:?}", self_id, p.as_ref().display(), err)
                }
            }
        }
        let yaml_string = files.join("\n");
        match serde_yaml::from_str(&yaml_string) {
            Ok(config) => {
                log::info!("{}.read | Reading configuration files - ok", self_id);
                AppConfig::from_yaml_value(&config)
            }
            Err(err) => {
                panic!("{}.read | Error in config: {:?}\n\terror: {:?}", self_id, yaml_string, err)
            }
        }
    }
}
