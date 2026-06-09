use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{LinkName, Services, conf::ConfDuration, entity::{Name, Point, ToPoint}, task::functions::{FnConfKind, FnConfPointType, FnConfig}};
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};
use indexmap::IndexMap;
use crate::{
    domain::FnOutRef,
    services::task::{
        functions::{*, functions::Functions},
        // FnAcc, FnAverage, FnConst, FnCount, FnDebug, FnEnable, FnHold, FnInput, FnIsChangedValue, FnMax, FnMin, FnPiecewiseLineApprox, FnPointId, FnRecOpCycleMetric, FnTimer, FnTimerOffDelay, FnTimerOnDelay, FnToBool, FnToDouble, FnVar, PiecewiseLinear, SqlMetric, 
        // functions::{
        //     comp::{FnEq, FnGe, FnGt, FnLe, FnLt, FnNe}, conversion::{FnToInt, FnToReal, FnToString},
        //     edge_detection::{FnFallingEdge, FnRisingEdge}, export::{FnExport, FnPoint, FnToApiQueue},
        //     filter::{FnSelect, FnSmooth, FnThreshold}, functions::Functions, io::FnRetain,
        //     ops::{FnAdd, FnBitAnd, FnBitOr, FnBitXor, FnDiv, FnMul, FnNot, FnPow, FnSub}, plot::FnPlot,
        // }
        task_nodes::TaskNodes
    },
};
///
/// Creates nested functions tree from it config
pub struct FnBuilder {}
//
impl FnBuilder {
    ///
    /// Creates nested functions tree from it config
    pub fn new(parent: &Name, tx_id: usize, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnOutRef, Error> {
        Self::function(parent, tx_id, "", conf, task_nodes, services)
        // trace!("{}.function | fn '{}': {:#?}", format!("{}/FnBuilder", parent), conf.borrow().id(), conf);
        // conf
    }
    fn get_input_config(txid: usize, parent: &Name, name: &str, conf: &mut FnConfig, task_nodes: &mut TaskNodes, services: &Arc<Services>) -> Result<Option<FnOutRef>, Error> {
        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
        Ok(match input_conf {
            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())?),
            None => None,
        })
    }
    ///
    ///
    fn function(parent: &Name, txid: usize, input_name: &str, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnOutRef, Error> {
        let dbg = Dbg::new(parent, "FnBuilder");
        let error = Error::new(&dbg, "function");
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
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnCount | Can't get 'initial'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnCount | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnCount::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Add => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAdd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnAdd::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Timer => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'reset'"), err))?;
                        let initial = Self::get_input_config(txid, parent, "initial", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'initial'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnTimer | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(FnTimer::new(parent, initial, reset, input), task_nodes.enable_mode(), enable))),
                            None => Rc::new(RefCell::new(FnTimer::new(parent, initial, reset, input))),
                        })
                    }
                    //
                    Functions::TimerOnDelay => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnTimerOnDelay | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get 'input'"), err))?;
                        let delay = {
                            let name = "delay";
                            let param = conf.param(name).ok_or(error.err(format!("FnTimerOnDelay | Can't get '{name}'")))?;
                            let delay = param.as_param().conf;
                            let delay = delay.as_str()
                                .ok_or(error.err(format!("FnTimerOnDelay | Wrong conf in '{name}': '{:?}'", param)))?;
                            ConfDuration::from_str(delay)
                                .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Wrong conf in '{name}': {:?}", param), err))?
                        };
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnTimerOnDelay::new(parent, reset, delay, input), task_nodes.enable_mode(), enable))),
                            None => Rc::new(RefCell::new(FnTimerOnDelay::new(parent, reset, delay, input))),
                        })
                    }
                    //
                    Functions::TimerOffDelay => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnTimerOffDelay | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get 'input'"), err))?;
                        let delay = {
                            let name = "delay";
                            let param = conf.param(name).ok_or(error.err(format!("FnTimerOffDelay | Can't get '{name}'")))?;
                            let delay = param.as_param().conf;
                            let delay = delay.as_str()
                                .ok_or(error.err(format!("FnTimerOffDelay | Wrong conf in '{name}': '{:?}'", param)))?;
                            ConfDuration::from_str(delay)
                                .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Wrong conf in '{name}': {:?}", param), err))?
                        };
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnTimerOffDelay::new(parent, reset, delay, input), task_nodes.enable_mode(), enable))),
                            None => Rc::new(RefCell::new(FnTimerOffDelay::new(parent, reset, delay, input))),
                        })
                    }
                    //
                    Functions::ToApiQueue => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes ,services.clone())
                            .map_err(|err| error.pass_with(format!("FnToApiQueue | Can't get '{name}'"), err))?;
                        let Some(queue_name) = conf.param("queue").map(|v| v.as_param()) else {
                            return Err(error.err(format!("FnToApiQueue | Parameter 'queue' is missed in '{}'", conf.name)));
                        };
                        let queue_name = queue_name.conf.as_str().unwrap();
                        let link_name = LinkName::from_str(queue_name).unwrap();
                        let send_queue = services.get_link(&link_name)
                            .map_err(|err| error.pass_with(format!("FnToApiQueue | Can't get link '{link_name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToApiQueue::new(parent, input, send_queue)
                        )))
                    }
                    //
                    Functions::Gt => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnGt | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnGt::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Ge => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnGe | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnGe::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Eq => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnEq | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnEq::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Le => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnLe | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnLe::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Lt => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnLt | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnLt::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Ne => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnNe | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnNe::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::SqlMetric => {
                        Ok(Rc::new(RefCell::new(
                            SqlMetric::new(parent, conf, task_nodes, services)
                                .map_err(|err| error.pass_with(format!("Can't build SqlMetric"), err))?
                        )))
                    }
                    //
                    Functions::PointId => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnPointId | Can't get '{name}'"), err))?;
                        // debug!("{}.functions | Functions::PointId | input: {:?}", self_id, input);
                        log::debug!("{}.functions | Functions::PointId | requesting points...", dbg);
                        let points = services.points(&parent.join())
                            .then(|points| points, |err| {
                                log::error!("{}.functions | Functions::PointId | Requesting points error: {:?}", dbg, err);
                                vec![]
                            });
                        // debug!("{}.functions | Functions::PointId | points: {:?}", self_id, points);
                        Ok(Rc::new(RefCell::new(
                            FnPointId::new(parent, input, points)
                        )))
                    }
                    //
                    Functions::Debug => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnDebug | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnDebug::new(parent, inputs)
                        )))
                    }
                    Functions::Plot => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "x";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let x = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let mut inputs = IndexMap::new();
                        let mut conf_inputs: IndexMap<String, FnConfKind> = conf.inputs
                            .iter()
                            .filter(|(name, _)| ! ["enable", "x"].contains(&(name.as_str())))
                            .map(|(n, c)| (n.to_owned(), c.clone())).collect();
                        for (name, input_conf) in &mut conf_inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?;
                            inputs.insert(name.to_owned(), input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnPlot::new(parent, enable, x, inputs)
                        )))

                    }
                    //
                    Functions::ToBool => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToBool | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToBool::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToInt => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToInt | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToInt::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToReal => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToReal | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToReal::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToDouble => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToDouble | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToDouble::new(parent, input)
                        )))
                    }
                    //
                    Functions::Export => {
                        let name = "input";
                        let input_conf = conf.input_conf(name)
                            .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?;
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?;
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "conf";
                        let point_conf = match conf.input_conf(name) {
                            Ok(FnConfKind::PointConf(p_conf)) => Some(p_conf.conf.clone()),
                            Ok(_) => return Err(error.err(format!("FnExport | Invalid Point config in '{name}'"))),
                            Err(_) => None,
                        };
                        let send_queue = match conf.param("send-to") {
                            Some(FnConfKind::Param(queue_name)) => {
                                let queue_name = queue_name.conf.as_str()
                                    .ok_or(error.err(format!("FnExport | Invalid conf in 'send-to', string expected")))?;
                                let link_name = LinkName::from_str(queue_name).unwrap();
                                services.get_link(&link_name).map_or(None, |send| Some(send))
                            }
                            Some(_) => return Err(error.err(format!("FnExport | Invalid conf in 'send-to'"))),
                            None => {
                                log::warn!("{}.function | FnExport | Parameter 'send-to' - missed in '{}'", dbg, conf.name);
                                None
                            },
                        };
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnExport::new(parent, point_conf, input, send_queue), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnExport::new(parent, point_conf, input, send_queue))),
                        })
                    }
                    //
                    Functions::Select => {
                        let select = Self::get_input_config(txid, parent, "select", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnSelect | 'select' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'select'"), err))?;
                        let default = Self::get_input_config(txid, parent, "default", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'default'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnSelect | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSelect::new(parent, default, input, select)
                        )))
                    }
                    //
                    Functions::RisingEdge => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnRisingEdge | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnRisingEdge | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnRisingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::FallingEdge => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnFallingEdge | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnFallingEdge | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnFallingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::Retain => {
                        let name = "default";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let default = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRetain | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let input = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRetain | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRetain | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "every-cycle";
                        let every_cycle = conf.param(name).map_or(false, |param| {
                            match param.as_param().conf.as_bool() {
                                Some(param) => param,
                                None => {
                                    log::warn!("{}.function | FnRetain | Illegal 'every_cycle' parameter value in '{:#?}'", dbg, conf);
                                    false
                                },
                            }
                        });
                        let Some(key) = conf.param("key").map(|v| v.as_param()) else {
                            return Err(error.err(format!("FnRetain | Parameter 'key' - missed in '{}'", conf.name)));
                        };
                        let key = key.conf.as_str().ok_or(error.err(format!("FnRetain | Parameter 'key' must be a string in '{}'", conf.name)))?;
                        let Some(retain_path) = services.retain().path else {
                            return Err(error.err(format!("FnRetain | Retain: path - missed in Application config")));
                        };
                        Ok(Rc::new(RefCell::new(
                            FnRetain::new(parent, retain_path, enable, every_cycle, key, default, input)
                        )))
                    }
                    //
                    Functions::Acc => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAcc | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnAcc | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnAcc::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Mul => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnMul::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Div => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnDiv::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Sub => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnSub::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::BitAnd => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitAnd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitAnd::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::BitOr => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitOr | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitOr::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::BitXor => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitXor | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitXor::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    Functions::Or => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnOr | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnOr::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    Functions::And => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAnd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnOr::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Not => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnBitNot | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnNot::new(parent, input)
                        )))
                    }
                    //
                    Functions::Threshold => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'enable'"), err))?;
                        let threshold = Self::get_input_config(txid, parent, "threshold", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnThreshold | 'threshold' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'threshold'"), err))?;
                        let factor = Self::get_input_config(txid, parent, "factor", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'factor'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnThreshold | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnThreshold::new(parent, threshold, factor, input),
                                task_nodes.enable_mode(),
                                enable
                            ))),
                            None => Rc::new(RefCell::new(FnThreshold::new(parent, threshold, factor, input))),
                        })
                    }
                    //
                    Functions::Smooth => {
                        let name = "factor";
                        let input_conf = conf.input_conf(name).unwrap();
                        let factor = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnSmooth | Can't get '{name}'"), err))?;
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnSmooth | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSmooth::new(parent, factor, input)
                        )))
                    }
                    //
                    Functions::Average => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAverage | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "reset";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let reset = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnMax | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnAverage | Can't get '{name}'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnAverage::new(parent, reset, input), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnAverage::new(parent, reset, input))),
                        })
                    }
                    //
                    Functions::Pow => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnPow::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::RecOpCycleMetric => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get 'reset'"), err))?;
                        let send_to = match conf.param("send-to") {
                            Some(queue_name) => {
                                let queue_name = match queue_name {
                                    FnConfKind::Param(queue_name) => queue_name.conf.as_str().unwrap(),
                                    _ => Err(error.err(format!("FnRecOpCycleMetric | Parameter 'send-to' - invalid type, string expected) '{:?}'", queue_name)))?,
                                };
                                let link_name = LinkName::from_str(queue_name).unwrap();
                                services.get_link(&link_name).map_or(None, |send| Some(send))
                            }
                            None => {
                                log::warn!("{}.function | FnRecOpCycleMetric | Parameter 'send-to' - missed in '{}'", dbg, conf.name);
                                None
                            },
                        };
                        let name = "op-cycle";
                        let op_cycle = Self::get_input_config(txid, parent, name, conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnRecOpCycleMetric | '{name}' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get '{name}'"), err))?;
                        let mut inputs = IndexMap::new();
                        let conf_inputs = conf.inputs.iter_mut().filter(|(name, _)| {
                            ! ["enable", "reset", "send-to", "conf", "op-cycle"].contains(&name.as_str())
                        });
                        for (name, input_conf) in conf_inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get '{name}'"), err))?;
                            inputs.insert(name.to_owned(), input);
                        }
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(
                                FnRecOpCycleMetric::new(parent, send_to, reset, op_cycle, inputs),
                                task_nodes.enable_mode(),
                                en,
                            ))),
                            None => Rc::new(RefCell::new(FnRecOpCycleMetric::new(parent, send_to, reset, op_cycle, inputs))),
                        })
                    }
                    //
                    Functions::Max => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnMax | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnMax::new(parent, reset, input), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnMax::new(parent, reset, input))),
                        })
                    }
                    //
                    Functions::Min => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnMin | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnMin::new(parent, reset, input), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnMin::new(parent, reset, input))),
                        })
                    }
                    //
                    Functions::PiecewiseLineApprox => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnPiecewiseLineApprox | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnPiecewiseLineApprox | Can't get 'input'"), err))?;
                        log::trace!("{}.function | PiecewiseLineApprox | conf: {:#?}", dbg, conf);
                        let name = "piecewise";
                        let FnConfKind::Param(piecewise) = conf.param(name)
                            .ok_or(error.err(format!("FnPiecewiseLineApprox | Can't get '{name}'")))? else {
                                return Err(error.err(format!("FnPiecewiseLineApprox | Parameter 'piecewise' - has invalid type, expected map in '{}'", conf.name)));
                            };
                        let pieces = PiecewiseLinear::from_yaml(parent, &piecewise.conf)
                            .map_err(|err| error.pass_with(format!("FnPiecewiseLineApprox | Wrong conf in '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnPiecewiseLineApprox::new(parent, input, pieces)
                        )))
                    }
                    //
                    Functions::IsChangedValue => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnIsChangedValue | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnIsChangedValue::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::Hold => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnHold | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnHold | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnHold::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToString => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnToString | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnToString | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToString::new(parent, input)
                        )))
                    }
                    //
                    // Add a new function here...
                    _ => Err(error.err(format!("Unknown function name: {:?}", conf.name))),
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
                            Self::function(parent, txid, input_conf_name, input_conf, task_nodes, services)
                                .map_err(|err| error.pass_with(format!("Var | Can't get '{input_conf_name}'"), err))?,
                        );
                        log::trace!("{}.function | Var: {:?}: {:?}", dbg, &conf.name, var.clone());
                        task_nodes.add_var(conf.name.clone(), var.clone())
                            .map_err(|err| error.pass_with(format!("Var | Can't add var '{}'", conf.name), err))?;
                        // log::debug!("{}.function | Var: {:?}", input);
                        Ok(var)
                    }
                    // Usage declared variable
                    None => {
                        let var = task_nodes.get_var(&var_name)
                            .ok_or(error.err(format!("Var {var_name} - is not declared")))?
                            .to_owned();
                        // let var = nodeVar.var();
                        task_nodes.add_var_out(conf.name.clone())
                            .map_err(|err| error.pass_with(format!("Var | Can't add defined var '{}'", conf.name), err))?;
                        Ok(var)
                    }
                }
            }
            FnConfKind::Const(conf) => {
                let value = conf.name.trim().to_lowercase();
                let name = format!("const {:?} '{}'", conf.type_, value);
                log::trace!("{}.function | Const: {:?}...", dbg, name);
                let value = match conf.type_.clone() {
                    FnConfPointType::Bool => value.parse::<bool>().unwrap().to_point(txid, &name),
                    FnConfPointType::Int => value.parse::<i64>().unwrap().to_point(txid, &name),
                    FnConfPointType::Real => value.parse::<f32>().unwrap().to_point(txid, &name),
                    FnConfPointType::Double => value.parse::<f64>().unwrap().to_point(txid, &name),
                    FnConfPointType::String => value.to_point(txid, &name),
                    FnConfPointType::Any => Err(error.err(format!("Const '{name}': type 'any' - is not supported")))?,
                    FnConfPointType::Unknown => Err(error.err(format!("Const '{name}': type required")))?,
                };
                let fn_const = Self::fn_const(&name, value);
                // taskNodes.addInput(inputName, input.clone());
                log::trace!("{}.function | Const: {:?} - done", dbg, fn_const);
                Ok(fn_const)
            }
            FnConfKind::Point(conf) => {
                log::trace!("{}.function | Input (Point<{:?}>): {:?} ({:?})...", dbg, conf.type_, input_name, conf.name);
                let point_name = conf.name.clone();
                let input = task_nodes.add_input(
                    &point_name,
                    Rc::new(RefCell::new(
                        FnInput::new(&point_name, txid, conf, &task_nodes.cycle())
                    )),
                );
                log::trace!("{}.function | input (Point): {:?}", dbg, input);
                input
            }
            FnConfKind::PointConf(conf) => {
                let send_to = match conf.send_to.as_ref() {
                    Some(send_to) => {
                        let link_name = LinkName::from_str(send_to).unwrap();
                        Some(services.get_link(&link_name)
                            .map_err(|err| error.pass_with(format!("PointConf | Can't get link"), err))?)
                    }
                    None => None,
                };
                let enable = match conf.enable.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "enable", input_conf, task_nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'enable'"), err))?),
                    None => None,
                };
                let input = match conf.input.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "input", input_conf, task_nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'input'"), err))?),
                    None => None,
                };
                let changes_only = match conf.changes_only.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "changes-only", input_conf, task_nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'input'"), err))?),
                    None => None,
                };
                Ok(Rc::new(RefCell::new(
                    FnPoint::new(parent, conf.conf.clone(), enable, changes_only, input, send_to),
                )))
            }
            FnConfKind::Param(conf) => {
                Err(error.err(format!("Param | Undefined variable or unknown custom parameters in the function conf: {:#?}", conf)))
            }
        }
    }
    ///
    ///
    fn fn_var(parent: impl Into<String>, input: FnOutRef,) -> FnOutRef {
        Rc::new(RefCell::new(
        FnVar::new(parent, input),
        ))
    }
        ///
    ///
    fn fn_const(parent: &str, value: Point) -> FnOutRef {
        Rc::new(RefCell::new(
        FnConst::new(parent, value)
        ))
    }
}
