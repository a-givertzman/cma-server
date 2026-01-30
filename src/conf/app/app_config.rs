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
        let name: String = conf.get("name").expect(&format!("AppConfig.new | 'name' - not found or wrong format"));
        let name = Name::new("", name);
        let dbg = format!("AppConfig({})", name);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let description = conf.get("description").unwrap_or("".into());
        log::trace!("{}.new | description: {:?}", dbg, description);
        let thread_pool = conf.get("thread-pool").map(|v: u64| v as usize);
        log::trace!("{}.new | thread-pool: {:?}", dbg, thread_pool);
        let services = conf.get("services").expect(&format!("{dbg}.new | 'services' - not found or wrong config"));
        let services = ServicesConf::new(&dbg, services);
        log::trace!("{}.new | services: {:#?}", dbg, services);
        // let services = RetainConf::default();
        let nodes: IndexMap<ConfKeywd, ConfTree> = conf.nodes()
            .filter(|node| !["name", "description", "services", "retain", "thread-pool"].contains(&node.key.as_str()))
            .map(|node| {
                let keyword = ConfKeywd::from_str(&node.key).expect(&format!("{dbg}.new | Can't parse keyword '{:?}'", node.key));
                match keyword.kind() {
                    k if k == ConfKind::Service.to_string() || k == ConfKind::Task.to_string() => {
                        let node_name = keyword.name();
                        if log::max_level() == log::LevelFilter::Debug {
                            let sufix = match keyword.title().is_empty() {
                                true => "".to_owned(),
                                false => format!(": '{}'", keyword.title()),
                            };
                            log::debug!("{}.new | service '{}'{}", dbg, node_name, sufix);
                        } else if log::max_level() == log::LevelFilter::Trace {
                            log::trace!("{}.new | DB '{}'   |   conf: {:?}", dbg, node_name, node);
                        }
                        (
                            keyword,
                            node,
                        )
                    }
                    _ => {
                        panic!("{}.new | Unknown node '{:?}' - is the root of the application config", dbg, keyword);
                    }
                }
            }).collect();
        Self {
            name,
            description,
            tread_pool: thread_pool,
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
        let dbg = "AppConfig";
        log::info!("{dbg}.read | Reading configuration files...");
        let conf = path.iter().fold(String::new(), |conf, p| {
            match fs::read_to_string(&p) {
                Ok(f) => {
                    log::info!("{dbg}.read | \t '{}' - Ok", p.as_ref().display());
                    format!("{conf}\n{f}")
                }
                Err(err) => {
                    log::error!("{dbg}.read | Can't read config file '{}', error: {:?}", p.as_ref().display(), err);
                    conf
                }
            }
        });
        match serde_yaml::from_str(&conf) {
            Ok(conf) => {
                log::info!("{dbg}.read | Reading configuration files - Ok");
                AppConfig::from_yaml_value(&conf)
            }
            Err(err) => {
                panic!("{dbg}.read | Can't parse config yaml: {}\n\t parse error: {:?}", conf, err)
            }
        }
    }
}
