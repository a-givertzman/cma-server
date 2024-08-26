use std::{cell::RefCell, rc::Rc, str::FromStr, sync::{Arc, RwLock}};
use indexmap::IndexMap;
use log::{debug, error, trace, warn};
use crate::{
    conf::{fn_::{fn_conf_keywd::FnConfPointType, fn_conf_kind::FnConfKind}, point_config::name::Name},
    core_::{
        point::point::{Point, ToPoint},
        types::fn_in_out_ref::FnInOutRef,
    },
    services::{
        queue_name::QueueName, safe_lock::SafeLock, services::Services, task::{
            nested_function::{
                comp::{fn_eq::FnEq, fn_ge::FnGe, fn_gt::FnGt, fn_le::FnLe, fn_lt::FnLt, fn_ne::FnNe}, edge_detection::{fn_falling_edge::FnFallingEdge, fn_rising_edge::FnRisingEdge}, export::{fn_export::FnExport, fn_point::FnPoint, fn_to_api_queue::FnToApiQueue}, filter::{fn_filter::FnFilter, fn_smooth::FnSmooth, fn_threshold::FnThreshold}, fn_acc::FnAcc, fn_average::FnAverage, fn_const::FnConst, fn_count::FnCount, fn_debug::FnDebug, fn_input::FnInput, fn_is_changed_value::FnIsChangedValue, fn_keep_valid::FnKeepValid, fn_max::FnMax, fn_piecewise_line_approx::FnPiecewiseLineApprox, fn_point_id::FnPointId, fn_rec_op_cycle_metric::FnRecOpCycleMetric, fn_timer::FnTimer, fn_to_bool::FnToBool, fn_to_double::FnToDouble, fn_to_int::FnToInt, fn_to_real::FnToReal, fn_to_string::FnToString, fn_var::FnVar, functions::Functions, io::fn_retain::FnRetain, ops::{
                    fn_add::FnAdd, fn_bit_and::FnBitAnd, fn_bit_not::FnBitNot, fn_bit_or::FnBitOr, fn_bit_xor::FnBitXor, fn_div::FnDiv, fn_mul::FnMul, fn_pow::FnPow, fn_sub::FnSub 
                }, plot::fn_plot::FnPlot, sql_metric::SqlMetric
            },
            task_nodes::TaskNodes,
        }
    },
};
///
/// Creates nested functions tree from it config
pub struct NestedFn {}
impl NestedFn {
    ///
    /// Creates nested functions tree from it config
    pub fn new(parent: &Name, tx_id: usize, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<RwLock<Services>>) -> FnInOutRef {
        Self::function(parent, tx_id, "", conf, task_nodes, services)
        // trace!("{}.function | fn '{}': {:#?}", format!("{}/NestedFn", parent), conf.borrow().id(), conf);
        // conf
    }
    ///
    ///
    fn function(parent: &Name, tx_id: usize, input_name: &str, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<RwLock<Services>>) -> FnInOutRef {
        let self_id = format!("{}/NestedFn", parent);
        match conf {
            FnConfKind::Fn(conf) => {
                trace!("{}.function | Fn {:?}: {:?}...", self_id, input_name, conf.name.clone());
                let c = conf.name.clone();
                let fn_name= c.clone();
                let fn_name = fn_name.as_str();
                drop(c);
                let fn_name = Functions::from_str(fn_name).unwrap();
                trace!("{}.function | Fn '{}' detected", self_id, fn_name.name());
                trace!("{}.function | fn_conf: {:?}: {:#?}", self_id, conf.name, conf);
                match fn_name {
                    //
                    Functions::Count => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnCount::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Add => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.push(input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnAdd::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::Timer => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, tx_id, name, conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnTimer::new(parent, enable, initial, input, true)
                        )))
                    }
                    //
                    Functions::ToApiQueue => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes ,services.clone());
                        let queue_name = conf.param("queue").unwrap_or_else(|_|
                            panic!("{}.function | Parameter 'queue' - missed in '{}'", self_id, conf.name)
                        ).as_param();
                        let queue_name = queue_name.conf.as_str().unwrap();
                        let send_queue = {
                            let services_lock = services.rlock(&self_id);
                            services_lock.get_link(&QueueName::new(queue_name)).unwrap_or_else(|err| {
                            panic!("{}.function | services.get_link error: {:#?}", self_id, err);
                            })
                        };
                        Rc::new(RefCell::new(Box::new(
                            FnToApiQueue::new(parent, input, send_queue)
                        )))
                    }
                    //
                    Functions::Gt => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnGt::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Ge => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnGe::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Eq => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnEq::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Le => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnLe::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Lt => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnLt::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Ne => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnNe::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::SqlMetric => {
                        Rc::new(RefCell::new(Box::new(
                            SqlMetric::new(parent, conf, task_nodes, services)
                        )))
                    }
                    //
                    Functions::PointId => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        // debug!("{}.functions | Functions::PointId | input: {:?}", self_id, input);
                        debug!("{}.functions | Functions::PointId | requesting points...", self_id);
                        let points = services.rlock(&format!("{}.PointId", self_id)).points(&parent.join())
                            .then(|points| points, |err| {
                                error!("{}.functions | Functions::PointId | Requesting points error: {:?}", self_id, err);
                                vec![]
                            });
                        // debug!("{}.functions | Functions::PointId | points: {:?}", self_id, points);
                        Rc::new(RefCell::new(Box::new(
                            FnPointId::new(parent, input, points)
                        )))
                    }
                    //
                    Functions::Debug => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.push(input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnDebug::new(parent, inputs)
                        )))
                    }
                    Functions::Plot => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "x";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let x = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let mut inputs = IndexMap::new();
                        let mut conf_inputs: IndexMap<String, FnConfKind> = conf.inputs
                            .iter()
                            .filter(|(name, _)| ! ["enable", "x"].contains(&(name.as_str())))
                            .map(|(n, c)| (n.to_owned(), c.clone())).collect();
                        for (name, input_conf) in &mut conf_inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.insert(name.to_owned(), input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnPlot::new(parent, enable, x, inputs)
                        )))

                    }
                    //
                    Functions::ToBool => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToBool::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToInt => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToInt::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToReal => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToReal::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToDouble => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToDouble::new(parent, input)
                        )))
                    }
                    //
                    Functions::Export => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let point_conf = match conf.input_conf("conf") {
                            Ok(conf) => {
                                match conf {
                                    FnConfKind::PointConf(conf) => Some(conf.conf.clone()),
                                    _ => panic!("{}.function | Invalid Point config in: {:?}", self_id, conf.name()),
                                }
                            }
                            Err(_) => None,
                        };
                        let send_queue = match conf.param("send-to") {
                            Ok(queue_name) => {
                                let queue_name = match queue_name {
                                    FnConfKind::Param(queue_name) => queue_name.conf.as_str().unwrap(),
                                    _ => panic!("{}.function | Parameter 'send-to' - invalid type (string expected) '{:#?}'", self_id, queue_name),
                                };
                                {
                                    let services_lock = services.rlock(&self_id);
                                    services_lock.get_link(&QueueName::new(queue_name)).map_or(None, |send| Some(send))
                                }
                            }
                            Err(_) => {
                                warn!("{}.function | Parameter 'send-to' - missed in '{}'", self_id, conf.name);
                                None
                            },
                        };
                        Rc::new(RefCell::new(Box::new(
                            FnExport::new(parent, enable, point_conf, input, send_queue)
                        )))
                    }
                    //
                    Functions::Filter => {
                        let name = "default";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let default = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "pass";
                        let input_conf = conf.input_conf(name).unwrap();
                        let pass = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnFilter::new(parent, default, input, pass)
                        )))
                    }
                    //
                    Functions::RisingEdge => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnRisingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::FallingEdge => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnFallingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::Retain => {
                        let name = "default";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let default = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let input = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "every-cycle";
                        let every_cycle = conf.param(name).map_or(false, |param| {
                            match param.as_param().conf.as_bool() {
                                Some(param) => param,
                                None => {
                                    warn!("{}.function | Illegal 'every_cycle' parameter value in '{:#?}'", self_id, conf);
                                    false
                                },
                            }
                        });
                        let key = conf.param("key").unwrap_or_else(|_|
                            panic!("{}.function | Parameter 'key' - missed in '{}'", self_id, conf.name)
                        ).as_param();
                        let key = key.conf.as_str().unwrap();
                        Rc::new(RefCell::new(Box::new(
                            FnRetain::new(parent, enable, every_cycle, key, default, input)
                        )))
                    }
                    //
                    Functions::Acc => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnAcc::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Mul => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnMul::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Div => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnDiv::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Sub => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnSub::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::BitAnd => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.push(input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnBitAnd::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::BitOr => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.push(input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnBitOr::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::BitXor => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.push(input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnBitXor::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::BitNot => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnBitNot::new(parent, input)
                        )))
                    }
                    //
                    Functions::Threshold => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "threshold";
                        let input_conf = conf.input_conf(name).unwrap();
                        let threshold = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "factor";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let factor = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnThreshold::new(parent, enable, threshold, factor, input)
                        )))
                    }
                    //
                    Functions::Smooth => {
                        let name = "factor";
                        let input_conf = conf.input_conf(name).unwrap();
                        let factor = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnSmooth::new(parent, factor, input)
                        )))
                    }
                    //
                    Functions::Average => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnAverage::new(parent, enable, input)
                        )))
                    }
                    //
                    Functions::Pow => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, tx_id, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnPow::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::RecOpCycleMetric => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let tx_send = match conf.param("send-to") {
                            Ok(queue_name) => {
                                let queue_name = match queue_name {
                                    FnConfKind::Param(queue_name) => queue_name.conf.as_str().unwrap(),
                                    _ => panic!("{}.function | Parameter 'send-to' - invalid type (string expected) '{:#?}'", self_id, queue_name),
                                };
                                {
                                    let services_lock = services.rlock(&self_id);
                                    services_lock.get_link(&QueueName::new(queue_name)).map_or(None, |send| Some(send))
                                }
                            }
                            Err(_) => {
                                warn!("{}.function | Parameter 'send-to' - missed in '{}'", self_id, conf.name);
                                None
                            },
                        };
                        let name = "op-cycle";
                        let input_conf = conf.input_conf(name).unwrap();
                        let op_cycle = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        let mut inputs = IndexMap::new();
                        let conf_inputs = conf.inputs
                            .iter_mut()
                            .filter(|(name, _)| {
                                ! ["enable", "send-to", "conf", "op-cycle"].contains(&name.as_str())
                            });
                        for (name, input_conf) in conf_inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.insert(name.to_owned(), input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnRecOpCycleMetric::new(parent, enable, tx_send, op_cycle, inputs)
                        )))
                    }
                    //
                    Functions::Max => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnMax::new(parent, enable, input)
                        )))
                    }
                    //
                    Functions::PiecewiseLineApprox => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        trace!("{}.function | PiecewiseLineApprox conf: {:#?}", self_id, conf);
                        let pieces: IndexMap<serde_yaml::Value, serde_yaml::Value> = match conf.param("piecewise") {
                            Ok(piecewise) => {
                                match piecewise {
                                    FnConfKind::Param(piecewise) => {
                                        serde_yaml::from_value(piecewise.conf.clone()).unwrap()
                                    }
                                    _ => panic!("{}.function | Parameter 'piecewise' - has invalid type (map expected) in '{}'", self_id, conf.name)
                                }
                            }
                            Err(_) => panic!("{}.function | Parameter 'piecewise' - missed in '{}'", self_id, conf.name),
                        };
                        Rc::new(RefCell::new(Box::new(
                            FnPiecewiseLineApprox::new(parent, input, pieces)
                        )))
                    }
                    //
                    Functions::IsChangedValue => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                            inputs.push(input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnIsChangedValue::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::KeepValid => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnKeepValid::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToString => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, tx_id, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToString::new(parent, input)
                        )))
                    }
                    //
                    // Add a new function here...
                    _ => panic!("{}.function | Unknown function name: {:?}", self_id, conf.name)
                }
            }
            FnConfKind::Var(conf) => {
                let var_name = conf.name.clone();
                trace!("{}.function | Var: {:?}...", self_id, var_name);
                match conf.inputs.iter_mut().next() {
                    //
                    // New var declaration
                    Some((input_conf_name, input_conf)) => {
                        let var = Self::fn_var(
                            var_name,
                            Self::function(parent, tx_id, input_conf_name, input_conf, task_nodes, services),
                        );
                        trace!("{}.function | Var: {:?}: {:?}", self_id, &conf.name, var.clone());
                        task_nodes.add_var(conf.name.clone(), var.clone());
                        // debug!("{}.function | Var: {:?}", input);
                        var
                    }
                    // Usage declared variable
                    None => {
                        let var = match task_nodes.get_var(&var_name) {
                            Some(var) => var,
                            None => panic!("{}.function | Var {:?} - not declared", self_id, &var_name),
                        }.to_owned();
                        // let var = nodeVar.var();
                        task_nodes.add_var_out(conf.name.clone());
                        var
                    }
                }
            }
            FnConfKind::Const(conf) => {
                let value = conf.name.trim().to_lowercase();
                let name = format!("const {:?} '{}'", conf.type_, value);
                trace!("{}.function | Const: {:?}...", self_id, &name);
                let value = match conf.type_.clone() {
                    FnConfPointType::Bool => value.parse::<bool>().unwrap().to_point(tx_id, &name),
                    FnConfPointType::Int => value.parse::<i64>().unwrap().to_point(tx_id, &name),
                    FnConfPointType::Real => value.parse::<f32>().unwrap().to_point(tx_id, &name),
                    FnConfPointType::Double => value.parse::<f64>().unwrap().to_point(tx_id, &name),
                    FnConfPointType::String => value.to_point(tx_id, &name),
                    FnConfPointType::Any => panic!("{}.function | Const of type 'any' - not supported", self_id),
                    FnConfPointType::Unknown => panic!("{}.function | Point type required", self_id),
                };
                let fn_const = Self::fn_const(&name, value);
                // taskNodes.addInput(inputName, input.clone());
                trace!("{}.function | Const: {:?} - done", self_id, fn_const);
                fn_const
            }
            FnConfKind::Point(conf) => {
                trace!("{}.function | Input (Point<{:?}>): {:?} ({:?})...", self_id, conf.type_, input_name, conf.name);
                let point_name = conf.name.clone();
                let input = task_nodes.add_input(
                    &point_name,
                    Rc::new(RefCell::new(Box::new(
                        FnInput::new(&point_name, tx_id, conf)
                    ))),
                );
                trace!("{}.function | input (Point): {:?}", self_id, input);
                input
            }
            FnConfKind::PointConf(conf) => {
                let services_lock = services.rlock(&self_id);
                let send_to = match &conf.send_to {
                    Some(send_to) => {
                        Some(services_lock.get_link(&QueueName::new(send_to)).unwrap_or_else(|err| {
                            panic!("{}.function | services.get_link error: {:#?}", self_id, err);
                        }))
                    }
                    None => None,
                };
                let enable = match conf.enable.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, tx_id, "enable", input_conf, task_nodes, services.clone())),
                    None => None,
                };
                let input = match conf.input.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, tx_id, "input", input_conf, task_nodes, services.clone())),
                    None => None,
                };
                let changes_only = match conf.changes_only.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, tx_id, "changes-only", input_conf, task_nodes, services.clone())),
                    None => None,
                };
                Rc::new(RefCell::new(Box::new(
                    FnPoint::new(parent, conf.conf.clone(), enable, changes_only, input, send_to),
                )))
            }
            FnConfKind::Param(conf) => {
                panic!("{}.function | Undefined variable or unknown custom parameters in the function conf: {:#?}", self_id, conf);
            }
        }
    }
    ///
    ///
    fn fn_var(parent: impl Into<String>, input: FnInOutRef,) -> FnInOutRef {
        Rc::new(RefCell::new(Box::new(
            FnVar::new(parent, input),
        )))
    }
    ///
    ///
    fn fn_const(parent: &str, value: Point) -> FnInOutRef {
        Rc::new(RefCell::new(Box::new(
            FnConst::new(parent, value)
        )))
    }
}
