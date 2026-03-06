use sal_core::dbg::Dbg;
use sal_sync::services::{LinkName, Services, conf::ConfDuration, entity::{Name, Point, ToPoint}, task::functions::{FnConfKind, FnConfPointType}};
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};
use indexmap::IndexMap;
use crate::{
    domain::FnInOutRef,
    services::task::{
        FnAcc, FnAverage, FnConst, FnCount, FnDebug, FnInput, FnIsChangedValue, FnKeepValid, FnMax, FnPiecewiseLineApprox, FnPointId, FnRecOpCycleMetric, FnTimer, FnTimerOffDelay, FnTimerOnDelay, FnToBool, FnToDouble, FnVar, SqlMetric, functions::{
            comp::{FnEq, FnGe, FnGt, FnLe, FnLt, FnNe}, conversion::{FnToInt, FnToReal, FnToString},
            edge_detection::{FnFallingEdge, FnRisingEdge}, export::{FnExport, FnPoint, FnToApiQueue},
            filter::{FnFilter, FnSmooth, FnThreshold}, functions::Functions, io::FnRetain,
            ops::{FnAdd, FnBitAnd, FnBitNot, FnBitOr, FnBitXor, FnDiv, FnMul, FnPow, FnSub}, plot::FnPlot,
        }, task_nodes::TaskNodes
    },
};
///
/// Creates nested functions tree from it config
pub struct FnBuilder {}
//
impl FnBuilder {
    ///
    /// Creates nested functions tree from it config
    pub fn new(parent: &Name, tx_id: usize, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<Services>) -> FnInOutRef {
        Self::function(parent, tx_id, "", conf, task_nodes, services)
        // trace!("{}.function | fn '{}': {:#?}", format!("{}/FnBuilder", parent), conf.borrow().id(), conf);
        // conf
    }
    ///
    ///
    fn function(parent: &Name, txid: usize, input_name: &str, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<Services>) -> FnInOutRef {
        let dbg = Dbg::new(parent, "FnBuilder");
        match conf {
            FnConfKind::Fn(conf) => {
                log::trace!("{}.function | Fn {:?}: {:?}...", dbg, input_name, conf.name.clone());
                let c = conf.name.clone();
                let fn_name= c.clone();
                let fn_name = fn_name.as_str();
                drop(c);
                let fn_name = Functions::from_str(fn_name).unwrap();
                log::trace!("{}.function | Fn '{}' detected", dbg, fn_name.name());
                log::trace!("{}.function | fn_conf: {:?}: {:#?}", dbg, conf.name, conf);
                match fn_name {
                    //
                    Functions::Count => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnCount::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Add => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, txid, name, conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnTimer::new(parent, enable, initial, input, true)
                        )))
                    }
                    //
                    Functions::TimerOnDelay => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "delay";
                        let delay = conf.param(name).map(|param| {
                            let delay = param.as_param().conf;
                            let delay = delay.as_str().expect(&format!("{dbg}.function | Wrong conf in '{name}': '{:?}'", param));
                            ConfDuration::from_str(delay).expect(&format!("{dbg}.function | Wrong conf in '{name}': '{:?}'", param))
                        }).expect(&format!("{dbg}.function | '{name}' - is missed in {:#?}", conf));
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, txid, name, conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnTimerOnDelay::new(parent, enable, delay, input)
                        )))
                    }
                    //
                    Functions::TimerOffDelay => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "delay";
                        let delay = conf.param(name).map(|param| {
                            let delay = param.as_param().conf;
                            let delay = delay.as_str().expect(&format!("{dbg}.function | Wrong conf in '{name}': '{:?}'", param));
                            ConfDuration::from_str(delay).expect(&format!("{dbg}.function | Wrong conf in '{name}': '{:?}'", param))
                        }).expect(&format!("{dbg}.function | '{name}' - is missed in {:#?}", conf));
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, txid, name, conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnTimerOffDelay::new(parent, enable, delay, input)
                        )))
                    }
                    //
                    Functions::ToApiQueue => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes ,services.clone());
                        let queue_name = conf.param("queue").unwrap_or_else(||
                            panic!("{}.function | Parameter 'queue' - missed in '{}'", dbg, conf.name)
                        ).as_param();
                        let queue_name = queue_name.conf.as_str().unwrap();
                        let link_name = LinkName::from_str(queue_name).unwrap();
                        let send_queue = services.get_link(&link_name).unwrap_or_else(|err| {
                            panic!("{}.function | services.get_link error: {:#?}", dbg, err);
                        });
                        Rc::new(RefCell::new(Box::new(
                            FnToApiQueue::new(parent, input, send_queue)
                        )))
                    }
                    //
                    Functions::Gt => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnGt::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Ge => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnGe::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Eq => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnEq::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Le => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnLe::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Lt => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnLt::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Ne => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
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
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        // debug!("{}.functions | Functions::PointId | input: {:?}", self_id, input);
                        log::debug!("{}.functions | Functions::PointId | requesting points...", dbg);
                        let points = services.points(&parent.join())
                            .then(|points| points, |err| {
                                log::error!("{}.functions | Functions::PointId | Requesting points error: {:?}", dbg, err);
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
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "x";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let x = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let mut inputs = IndexMap::new();
                        let mut conf_inputs: IndexMap<String, FnConfKind> = conf.inputs
                            .iter()
                            .filter(|(name, _)| ! ["enable", "x"].contains(&(name.as_str())))
                            .map(|(n, c)| (n.to_owned(), c.clone())).collect();
                        for (name, input_conf) in &mut conf_inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToBool::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToInt => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToInt::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToReal => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToReal::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToDouble => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToDouble::new(parent, input)
                        )))
                    }
                    //
                    Functions::Export => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let point_conf = conf.input_conf("conf").map(|conf| {
                            if let FnConfKind::PointConf(conf) = conf {
                                return conf.conf.clone()
                            }
                            panic!("{}.function | Invalid Point config in: {:?}", dbg, conf.name())
                        }).ok();
                        let send_queue = match conf.param("send-to") {
                            Some(queue_name) => {
                                let queue_name = match queue_name {
                                    FnConfKind::Param(queue_name) => queue_name.conf.as_str().unwrap(),
                                    _ => panic!("{}.function | Parameter 'send-to' - invalid type (string expected) '{:#?}'", dbg, queue_name),
                                };
                                let link_name = LinkName::from_str(queue_name).unwrap();
                                services.get_link(&link_name).map_or(None, |send| Some(send))
                            }
                            None => {
                                log::warn!("{}.function | Parameter 'send-to' - missed in '{}'", dbg, conf.name);
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
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "pass";
                        let input_conf = conf.input_conf(name).unwrap();
                        let pass = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnFilter::new(parent, default, input, pass)
                        )))
                    }
                    //
                    Functions::RisingEdge => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnRisingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::FallingEdge => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnFallingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::Retain => {
                        let name = "default";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let default = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let input = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "every-cycle";
                        let every_cycle = conf.param(name).map_or(false, |param| {
                            match param.as_param().conf.as_bool() {
                                Some(param) => param,
                                None => {
                                    log::warn!("{}.function | Illegal 'every_cycle' parameter value in '{:#?}'", dbg, conf);
                                    false
                                },
                            }
                        });
                        let key = conf.param("key").unwrap_or_else(||
                            panic!("{}.function | Parameter 'key' - missed in '{}'", dbg, conf.name)
                        ).as_param();
                        let key = key.conf.as_str().unwrap();
                        let retain_path = services.retain().path.unwrap_or_else(|| panic!("{}.function | Retain: path - missed in Application config", dbg));
                        Rc::new(RefCell::new(Box::new(
                            FnRetain::new(parent, retain_path, enable, every_cycle, key, default, input)
                        )))
                    }
                    //
                    Functions::Acc => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnAcc::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Mul => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnMul::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Div => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnDiv::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::Sub => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnSub::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::BitAnd => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnBitNot::new(parent, input)
                        )))
                    }
                    //
                    Functions::Threshold => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "threshold";
                        let input_conf = conf.input_conf(name).unwrap();
                        let threshold = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "factor";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let factor = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnThreshold::new(parent, enable, threshold, factor, input)
                        )))
                    }
                    //
                    Functions::Smooth => {
                        let name = "factor";
                        let input_conf = conf.input_conf(name).unwrap();
                        let factor = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnSmooth::new(parent, factor, input)
                        )))
                    }
                    //
                    Functions::Average => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnAverage::new(parent, enable, input)
                        )))
                    }
                    //
                    Functions::Pow => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services);
                        Rc::new(RefCell::new(Box::new(
                            FnPow::new(parent, input1, input2)
                        )))
                    }
                    //
                    Functions::RecOpCycleMetric => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let send_to = match conf.param("send-to") {
                            Some(queue_name) => {
                                let queue_name = match queue_name {
                                    FnConfKind::Param(queue_name) => queue_name.conf.as_str().unwrap(),
                                    _ => panic!("{}.function | Parameter 'send-to' - invalid type (string expected) '{:#?}'", dbg, queue_name),
                                };
                                let link_name = LinkName::from_str(queue_name).unwrap();
                                services.get_link(&link_name).map_or(None, |send| Some(send))
                            }
                            None => {
                                log::warn!("{}.function | Parameter 'send-to' - missed in '{}'", dbg, conf.name);
                                None
                            },
                        };
                        let name = "op-cycle";
                        let input_conf = conf.input_conf(name).unwrap();
                        let op_cycle = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        let mut inputs = IndexMap::new();
                        let conf_inputs = conf.inputs
                            .iter_mut()
                            .filter(|(name, _)| {
                                ! ["enable", "send-to", "conf", "op-cycle"].contains(&name.as_str())
                            });
                        for (name, input_conf) in conf_inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                            inputs.insert(name.to_owned(), input);
                        }
                        Rc::new(RefCell::new(Box::new(
                            FnRecOpCycleMetric::new(parent, enable, send_to, op_cycle, inputs)
                        )))
                    }
                    //
                    Functions::Max => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnMax::new(parent, enable, input)
                        )))
                    }
                    //
                    Functions::PiecewiseLineApprox => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        log::trace!("{}.function | PiecewiseLineApprox conf: {:#?}", dbg, conf);
                        let pieces: IndexMap<serde_yaml::Value, serde_yaml::Value> = match conf.param("piecewise") {
                            Some(piecewise) => {
                                match piecewise {
                                    FnConfKind::Param(piecewise) => {
                                        serde_yaml::from_value(piecewise.conf.clone()).unwrap()
                                    }
                                    _ => panic!("{}.function | Parameter 'piecewise' - has invalid type (map expected) in '{}'", dbg, conf.name)
                                }
                            }
                            None => panic!("{}.function | Parameter 'piecewise' - missed in '{}'", dbg, conf.name),
                        };
                        Rc::new(RefCell::new(Box::new(
                            FnPiecewiseLineApprox::new(parent, input, pieces)
                        )))
                    }
                    //
                    Functions::IsChangedValue => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
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
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnKeepValid::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToString => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone());
                        Rc::new(RefCell::new(Box::new(
                            FnToString::new(parent, input)
                        )))
                    }
                    //
                    // Add a new function here...
                    _ => panic!("{}.function | Unknown function name: {:?}", dbg, conf.name)
                }
            }
            FnConfKind::Var(conf) => {
                let var_name = conf.name.clone();
                log::trace!("{}.function | Var: {:?}...", dbg, var_name);
                match conf.inputs.iter_mut().next() {
                    //
                    // New var declaration
                    Some((input_conf_name, input_conf)) => {
                        let var = Self::fn_var(
                            var_name,
                            Self::function(parent, txid, input_conf_name, input_conf, task_nodes, services),
                        );
                        log::trace!("{}.function | Var: {:?}: {:?}", dbg, &conf.name, var.clone());
                        task_nodes.add_var(conf.name.clone(), var.clone());
                        // debug!("{}.function | Var: {:?}", input);
                        var
                    }
                    // Usage declared variable
                    None => {
                        let var = match task_nodes.get_var(&var_name) {
                            Some(var) => var,
                            None => panic!("{}.function | Var {:?} - not declared", dbg, &var_name),
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
                log::trace!("{}.function | Const: {:?}...", dbg, &name);
                let value = match conf.type_.clone() {
                    FnConfPointType::Bool => value.parse::<bool>().unwrap().to_point(txid, &name),
                    FnConfPointType::Int => value.parse::<i64>().unwrap().to_point(txid, &name),
                    FnConfPointType::Real => value.parse::<f32>().unwrap().to_point(txid, &name),
                    FnConfPointType::Double => value.parse::<f64>().unwrap().to_point(txid, &name),
                    FnConfPointType::String => value.to_point(txid, &name),
                    FnConfPointType::Any => panic!("{}.function | Const of type 'any' - not supported", dbg),
                    FnConfPointType::Unknown => panic!("{}.function | Point type required", dbg),
                };
                let fn_const = Self::fn_const(&name, value);
                // taskNodes.addInput(inputName, input.clone());
                log::trace!("{}.function | Const: {:?} - done", dbg, fn_const);
                fn_const
            }
            FnConfKind::Point(conf) => {
                log::trace!("{}.function | Input (Point<{:?}>): {:?} ({:?})...", dbg, conf.type_, input_name, conf.name);
                let point_name = conf.name.clone();
                let input = task_nodes.add_input(
                    &point_name,
                    Rc::new(RefCell::new(Box::new(
                        FnInput::new(&point_name, txid, conf)
                    ))),
                );
                log::trace!("{}.function | input (Point): {:?}", dbg, input);
                input
            }
            FnConfKind::PointConf(conf) => {
                let send_to = match &conf.send_to {
                    Some(send_to) => {
                        let link_name = LinkName::from_str(send_to).unwrap();
                        Some(services.get_link(&link_name).unwrap_or_else(|err| {
                            panic!("{}.function | services.get_link error: {:#?}", dbg, err);
                        }))
                    }
                    None => None,
                };
                let enable = match conf.enable.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "enable", input_conf, task_nodes, services.clone())),
                    None => None,
                };
                let input = match conf.input.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "input", input_conf, task_nodes, services.clone())),
                    None => None,
                };
                let changes_only = match conf.changes_only.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "changes-only", input_conf, task_nodes, services.clone())),
                    None => None,
                };
                Rc::new(RefCell::new(Box::new(
                    FnPoint::new(parent, conf.conf.clone(), enable, changes_only, input, send_to),
                )))
            }
            FnConfKind::Param(conf) => {
                panic!("{}.function | Undefined variable or unknown custom parameters in the function conf: {:#?}", dbg, conf);
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
