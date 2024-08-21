#[cfg(test)]
// #[path = "./tests"]
mod tests;
mod core_;

use log::{info, debug, trace, warn};
use core_::nested_function::fn_config_type::FnConfigType;
use serde::{Deserialize, Deserializer, de::{self}};
use std::{env, collections::HashMap, str::FromStr, fmt::{Debug, self}};

use crate::core_::nested_function::fn_conf_keywd::FnConfKeywd;


#[derive(Debug)]
struct Config {
    nodeType: FnConfigType,
    services: HashMap<String, Box<dyn ServiceConfig>>,
}
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigVisitor;

        impl<'de> de::Visitor<'de> for ConfigVisitor {
            type Value = Config;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Config.deserialize | expecting Config")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                match FnConfKeywd::from_str(&v) {
                    Ok(keyword) => {
                        trace!("Config.deserialize | keyword parsed: {:?}", keyword);
                        // nodes.insert(key, Config::new(&conf, &Some(keyword.type_())));
                        Ok(Self::Value {
                            nodeType: FnConfigType::Unknown,
                            services: HashMap::new(),
                        })
                    }
                    Err(err) => {
                        // warn!("Config.deserialize | Unknown keyword: '{:?}' in the conf: {:?}", key, conf);
                        let msg = format!("Config.deserialize | possible input name: '{:?}'", v);
                        warn!("{}", msg);
                        // nodes.insert(key, Config::new(&conf, &None));
                        Err(de::Error::custom(msg))
                    }
                }
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                if let Some(key) = map.next_key()? {
                    let value: HashMap<String, > = map.next_value()?;
                    match FnConfKeywd::from_str(key) {
                        Ok(keyword) => {
                            trace!("Config.deserialize | keyword parsed: {:?}", keyword);
                            // nodes.insert(key, Config::new(&conf, &Some(keyword.type_())));
                            Ok(Self::Value {
                                nodeType: FnConfigType::Unknown,
                                services: HashMap::new(),
                            })
                        }
                        Err(err) => {
                            // warn!("Config.deserialize | Unknown keyword: '{:?}' in the conf: {:?}", key, conf);
                            let msg = format!("Config.deserialize | possible input name: '{:?}'", key);
                            warn!("{}", msg);
                            // nodes.insert(key, Config::new(&conf, &None));
                            Err(de::Error::custom(msg))
                        }
                    }

                    // debug!("Config.deserialize | key: {:?}\tvalue: {:?}", key, value);
                    // Ok(Self::Value {
                    //     nodeType: FnConfigType::Unknown,
                    //     services: HashMap::new(),
                    // })

                    // if let Some(_) = map.next_key::<&str>()? {
                    //     Err(de::Error::duplicate_field("name"))
                    // } else {
                    //     // let key: String = key;
                    // }
                } else {
                    Err(de::Error::missing_field("name"))
                }
            }
        }

        deserializer.deserialize_map(ConfigVisitor {})        
        
        // let input = deserializer.deserialize_map(ConfigVisitor {});
        // Ok(Self {
        //     nodeType: FnConfigType::Unknown,
        //     services: HashMap::new(),
        // })
    }
}

pub trait ServiceConfig: Debug {
    fn run(&self);
}

#[derive(Debug)]
struct ServiceCmaClientConfig {
    nodeType: FnConfigType,
    
}
impl ServiceConfig for ServiceCmaClientConfig {
    fn run(&self) {
        todo!()
    }
}
#[derive(Debug)]
struct ServiceApiClientConfig {
    nodeType: FnConfigType,
    
}
impl ServiceConfig for ServiceApiClientConfig {
    fn run(&self) {
        todo!()
    }
}


impl Config {
    ///
    /// 
    pub fn new(conf: &serde_yaml::Value, nodeType: &Option<FnConfigType>) -> () {

    }
    /// 
    /// 
    pub fn new_(conf: &serde_yaml::Value, nodeType: &Option<FnConfigType>) -> () {
        // let nodes: HashMap<String, Config> = if conf.is_mapping() {
        if conf.is_mapping() {
            trace!("FnConfig.new | IS MAP");
            match serde_yaml::from_value(conf.clone()) {
                Ok(map) => {
                    let map: HashMap<String, serde_yaml::Value> = map;
                    trace!("FnConfig.new | confMap: {:?}", map);
                    let mut nodes = HashMap::new();
                    for (key, conf) in map {
                        match FnConfKeywd::from_str(key.as_str()) {
                            Ok(keyword) => {
                                trace!("FnConfig.new | keyword parsed: {:?}", keyword);
                                nodes.insert(key, Config::new(&conf, &Some(keyword.type_())));
                            }
                            Err(err) => {
                                warn!("FnConfig.new | Unknown keyword: '{:?}' in the conf: {:?}", key, conf);
                                warn!("FnConfig.new | possible input name: '{:?}'", key);
                                nodes.insert(key, Config::new(&conf, &None));
                            }
                        }
                    }
                    nodes
                }
                Err(err) => {
                    panic!("Error in config: {:?}\n\terror: {:?}", conf, err);
                }
            }
        } else if conf.is_string() {
            trace!("FnConfig.new | IS STRING");
            match serde_yaml::from_value(conf.clone()) {
                Ok(confStr) => {
                    let confStr: String = confStr;
                    trace!("FnConfig.new | confStr: {:?}", confStr);
                    match FnConfKeywd::from_str(confStr.as_str()) {
                        Ok(keyword) => {
                            trace!("FnConfig.new | keyword parsed: {:?}", keyword);
                            // return Config { nodeType: keyword.type_(), services: HashMap::new() }
                            return ()
                        }
                        Err(err) => {
                            warn!("FnConfig.new | Unknown keyword in the conf: {:?}", conf);
                            warn!("FnConfig.new | possible VAR detected: '{:?}'", conf);
                            // return Config { nodeType: FnConfigType::Var, services: HashMap::new() }
                            return ()
                            // panic!("FnConfig.new | Unknown keyword: {:?}", confStr);
                        }
                    }
                }
                Err(err) => {
                    panic!("Error in config: {:?}\n\terror: {:?}", conf, err);
                }
            }
        } else {
            panic!("Unknown config type (String & Mapping supported): {:?}", conf);
        };
        let nodeType: FnConfigType = match nodeType {
            Some(nt) => nt.clone(),
            None => FnConfigType::Unknown,
        };
        // Config { nodeType: nodeType, services: nodes }
    }
    // fn parseNode(conf: &serde_yaml::Value) -> Config {
    // }
}


fn main() {
    env::set_var("RUST_LOG", "trace");  // off / error / warn / info / debug / trace
    // env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_BACKTRACE", "full");
    env_logger::init();

    info!("test_create_valid_fn");
    // let (initial, switches) = init_each();
    let test_data = [
        // serde_yaml::from_str(r#"
        //     input: const 177.3
        // "#),
        // serde_yaml::from_str(r#"
        //     input: point '/Path/Point.Name/'
        // "#),

        // r#"
        //     const 177.3
        // "#,
        // serde_yaml::from_str(r#"
        //     point '/Path/Point.Name/'
        // "#),
        // r#"
        //     fn functionName:
        // "#,
        // r#"
        //     fn SqlMetric:
        //         initial: const 0      # начальное значение
        // "#,

        // serde_yaml::from_str(r#"
        //     let newVar1:
        //         input1: const 177.3
        //         input2: point '/Path/Point.Name/'
        //         input3:
        //             fn Count:
        //                 inputConst1: const '13.5'
        //                 inputConst2: newVar1
        // "#),
        // serde_yaml::from_str(r#"
        //     fn SqlMetric:
        //         initial: const 0
        //         sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"    
        //         inputs:
        //             input1:
        //                 let VarName2:
        //                     input: 
        //                         fn functionName:
        //                             initial: VarName2
        //                             input: 
        //                                 fn functionName:
        //                                     input1: const someValue
        //                                     input2: point '/path/Point.Name/'
        //                                     input: 
        //                                         fn functionName:
        //                                             input: point '/path/Point.Name/'        
        // "#),
        r#"
        let API: 
            nodeType: 1
            services:
                - s1
                - s2
        "#,
    ];

    // for conf in test_data {
    //     let config = Config::new(&conf.unwrap(), &None);
    //     debug!("config: {:?}", config);
    // }

    for conf in test_data {
        let config: Config = serde_yaml::from_str(conf).unwrap();
    }

}
















































// fn main() {
//     env::set_var("RUST_LOG", "debug");  // off / error / warn / info / debug / trace
//     // env::set_var("RUST_BACKTRACE", "1");
//     env::set_var("RUST_BACKTRACE", "full");
//     env_logger::init();

//     info!("test_create_valid_fn");
//     // let (initial, switches) = init_each();
//     let test_data = [
//         (serde_yaml::from_str(r#"
//             let newVar1:
//                 input1:
//                     fn Count:
//                         inputConst1: const '13.5'
//                         inputConst2: newVar1
//         "#), 
//         FnConfig{ fnType: FnConfigType::Const, name: "".to_string(), inputs: HashMap::new() }),
//                     // input2:
//                     //     fn Count:
//                     //         inputConst1: const '13.5'
//                     //         inputConst2: const '13.5'
//     ];
//     for (value, target) in test_data {
//         debug!("test value: {:?}", value);
//         let conf: serde_yaml::Value = value.unwrap();
//         // let conf = test_data.get("/").unwrap();

//         debug!("value: {:?}   |   conf: {:?}   |   target: {:?}", "_", conf, target);
//         // let fnKeyword = FnConfigKeyword::from_str(conf.as_str().unwrap()).unwrap();
//         // debug!("\tfnKeyword: {:?}", fnKeyword);
//         let mut vars = vec![];
//         let fnConfig = FnConfig::new(&conf, &mut vars);
//         debug!("\tfnConfig: {:?}", fnConfig);
//         debug!("\tvars: {:?}", vars);
//         // assert_eq!(fnConfigType, target);
//     }

// }
