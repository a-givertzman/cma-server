#[cfg(test)]
// #[path = "./tests"]
mod tests;
mod core_;

use log::{info, debug, trace, warn};
use serde::Deserialize;
use std::{env, fmt::Debug, time::Duration, thread::{self, JoinHandle}, cell::RefCell};



#[derive(Debug, Deserialize)]
enum ServiceType {
    #[serde(rename = "serviceCMA:")]
    CmaClient(ServiceCmaClient),
    #[serde(rename = "serviceAPI:")]
    ApiClient(ServiceApiClient),
    #[serde(rename = "serviceTask:")]
    Task(ServiceTask),
}

pub trait Service: Debug {
    fn run(&mut self);
    // #[stable(feature = "rust1", since = "1.0.0")]
    fn join(&mut self);
}


#[derive(Debug, Deserialize)]
struct ServiceTask {
    cycle: u64,
    #[serde(skip_deserializing)]
    handle: Option<JoinHandle<()>>,
}
impl Service for ServiceTask {
    fn run(&mut self) {
        let cycle = Duration::from_millis(self.cycle);
        let thread_join_handle = thread::spawn(move || {
            loop {
                debug!("ServiceTask | loop with cycle time: {:?}", cycle);
                thread::sleep(cycle);
            }
        });
        self.handle = Some(thread_join_handle);
    }
    fn join(&mut self) {
        if let Some(handle) = self.handle.take(){
            handle.join().unwrap();
        }        
    }
}

#[derive(Debug, Deserialize)]
struct ServiceCmaClient {
    nodeType: String,
    address: String,
    cycle: u64,
    #[serde(skip_deserializing)]
    handle: Option<JoinHandle<()>>,
}
impl Service for ServiceCmaClient {
    fn run(&mut self) {
        let cycle = Duration::from_millis(self.cycle);
        let thread_join_handle = thread::spawn(move || {
            loop {
                debug!("ServiceCmaClient | loop with cycle time: {:?}", cycle);
                thread::sleep(cycle);
            }
        });
        self.handle = Some(thread_join_handle);
    }
    fn join(&mut self) {
        if let Some(handle) = self.handle.take(){
            handle.join().unwrap();
        }        
    }
}
#[derive(Debug, Deserialize)]
struct ServiceApiClient {
    nodeType: String,
    address: String,
    cycle: u64,
    #[serde(skip_deserializing)]
    handle: Option<JoinHandle<()>>,    
}
impl Service for ServiceApiClient {
    fn run(&mut self) {
        let cycle = Duration::from_millis(self.cycle);
        let thread_join_handle = thread::spawn(move || {
            loop {
                debug!("ServiceApiClient | loop with cycle time: {:?}", cycle);
                thread::sleep(cycle);
            }
        });
        self.handle = Some(thread_join_handle);
    }
    fn join(&mut self) {
        if let Some(handle) = self.handle.take(){
            handle.join().unwrap();
        }        
    }
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
        - serviceCMA:
            nodeType: API Client
            address: 127.0.0.1:8899
            cycle: 1000
        - serviceAPI:
            nodeType: API Client
            address: 127.0.0.1:8899
            cycle: 2000
        - serviceTask:
            cycle: 200
        "#,
    ];

    // for conf in test_data {
    //     let config = Config::new(&conf.unwrap(), &None);
    //     debug!("config: {:?}", config);
    // }
    let mut services = vec![];
    let config: Vec<serde_yaml::Value> = serde_yaml::from_str(&test_data[0]).unwrap();
    for conf in config {
        debug!("main | key: {:?}\t|\tconf: {:?}", "_", conf);
        let conf = format!("!{}", serde_yaml::to_string(&conf).unwrap());
        debug!("main | key: {:?}\t|\tconf: {:?}", "_", conf);
        let service: RefCell<Box<dyn Service>> = match serde_yaml::from_str(&conf).unwrap() {
            ServiceType::ApiClient(service) => {
                RefCell::new(Box::new(service))
            }
            ServiceType::CmaClient(service) => {
                RefCell::new(Box::new(service))
            }
            ServiceType::Task(service) => {
                RefCell::new(Box::new(service))
            }
            // _ => {}
        };
        service.borrow_mut().run();
        services.push(service);
    }

    for service in services {
        service.borrow_mut().join();
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
