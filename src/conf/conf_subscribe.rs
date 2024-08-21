use std::collections::HashMap;
use log::trace;
use crate::{core_::cot::cot::Cot, services::multi_queue::subscription_criteria::SubscriptionCriteria};
use super::point_config::{point_config::PointConfig, point_config_history::PointConfigHistory};
///
/// Service Configuration, to be subscribed on some service / services, by number of criterias
/// ------------------------------------------------------------------------------------------
/// subscibe: MultiQueue    # - broadcast suscription to the MultiQueue
/// ------------------------------------------------------------------------------------------
/// subscribe:
///     MultiQueue: {}      # - broadcast suscription to the MultiQueue
///     AnotherService: {}  # - broadcast suscription to the AnotherService
/// ------------------------------------------------------------------------------------------
/// subscibe: 
///     MultiQueue:         # - multicast subscription to the MultiQueue
///         Inf: []         #   - on all points having Cot::Inf
/// ------------------------------------------------------------------------------------------
/// subscribe: 
///     MultiQueue:                     # - multicast subscription to the MultiQueue
///         {cot: Inf, history: r}: []  #   - on all points having Cot::Inf and history::read
/// ------------------------------------------------------------------------------------------
/// subscribe: 
///     MultiQueue:                     # - multicast subscription to the MultiQueue
///         {history: rw}: []            #   - on all points having history::read or history::write and Cot::All
/// ------------------------------------------------------------------------------------------
/// subscibe:
///     MultiQueue:                         # - multicast subscription to the MultiQueue
///         Act: []                         #   - on all points having Cot::Act
///         {cot: Inf, history: r}:         #   - on concrete points having Cot::Inf and history::read
///             - /App/Service/Point.Name.1
///             - /App/Service/Point.Name.2
///     AnotherService:                     # - multicast subscription to the AnotherService
///         Inf: []                         #   - on all points having Cot::Inf
#[derive(Debug, Clone, PartialEq)]
pub struct ConfSubscribe {
    id: String,
    conf: serde_yaml::Value,
}
///
/// Creates new instance from yaml
impl ConfSubscribe {
    ///
    /// Creates new instance of ConfSubscribe
    pub fn new(conf: serde_yaml::Value) -> Self {
        Self {
            id: "ConfSubscribe".to_owned(),
            conf,
        }
    }
    ///
    /// Returns subscriptions based on the given points and number of configured criterias:
    ///     - HashMap:
    ///         - key - service id
    ///         - value - subscriptions
    pub fn with(&self, points: &[PointConfig]) -> HashMap<String, Option<Vec<SubscriptionCriteria>>> {
        if self.conf.is_string() {
            let service = self.conf.as_str().unwrap().to_owned();
            HashMap::from([
                (service, Some(vec![]))
            ])
        } else if self.conf.is_mapping() {
            match self.conf.as_mapping() {
                Some(conf) => {
                    conf.iter().fold(HashMap::new(), |mut subscriptions, (service, criterias_conf)| {
                        let service = service.as_str().unwrap();
                        let criterias = Criterias::new(&self.id, criterias_conf, points).build();
                        subscriptions
                            .entry(service.to_owned())
                            .and_modify(|entry| {
                                if let Some(points) = &criterias {
                                    match entry {
                                        Some(entry) => entry.append(&mut points.clone()),
                                        None => {
                                            entry.replace(points.clone());
                                        }
                                    };
                                }
                            })
                            .or_insert(criterias);
                        subscriptions
                    })
                }
                None => {
                    panic!("{}.new | Yaml Mapping expected, but not found: {:#?}", self.id, self.conf);
                }
            }
        } else {
            panic!("{}.new | Invalid Subscribe option format: {:#?}", self.id, self.conf);
        }
    }
    ///
    /// Reurns true if subscription conf is empty
    pub fn is_empty(&self) -> bool {
        self.conf.is_null()
    }
}

///         Inf: []         #   - on all points having Cot::Inf
///         {cot: Inf, history: r}: []  #   - on all points having Cot::Inf and history::read
///         {cot: Inf, history: r}:         #   - on concrete points having Cot::Inf and history::read
///             - /App/Service/Point.Name.1
///             - /App/Service/Point.Name.2
struct Criterias {
    id: String,
    conf: serde_yaml::Value,
    points: Vec<PointConfig>
}
//
// 
impl Criterias {
    fn new(parent: &str, conf: &serde_yaml::Value, points: &[PointConfig]) -> Self {
        Self {
            id: format!("{}/Criterias", parent),
            conf: conf.clone(),
            points: points.to_vec(),
        }
    }
    ///
    /// Returns SubscriptionCreteria's
    fn build(&self) -> Option<Vec<SubscriptionCriteria>> {
        match self.conf.as_mapping() {
            Some(conf) => {
                let mut points: Option<Vec<SubscriptionCriteria>> = None;
                if conf.is_empty() {
                    points = Some(vec![]);
                } else {
                    for (options, names) in conf {
                        let criterias = Self::build_criterias(&self.id, options, names, &self.points);
                        trace!("{}.build | criterias: {:#?}", self.id, criterias);
                        if let Some(mut criterias) = criterias {
                            points = points
                                .as_mut()
                                .map_or(Some(criterias.clone()), |v| {
                                    v.append(&mut criterias);
                                    Some(v.to_vec())
                                });
                        }
                        trace!("{}.build | points: {:?}", self.id, points);
                    }
                }
                points
            }
            None => None,
        }
    }
    ///
    /// Creates list of Subscriptions based on the given point names, point configs, and filtering criterias
    fn build_criterias(self_id: &str, options: &serde_yaml::Value, names: &serde_yaml::Value, point_configs: &[PointConfig]) -> Option<Vec<SubscriptionCriteria>> {
        trace!("{}.build_criterias | options: {:?}", self_id, options);
        trace!("{}.build_criterias | names: {:?}", self_id, names);
        let names = names.as_sequence().unwrap();
        if options.is_string() {
            let cot: Cot = serde_yaml::from_value(options.clone()).unwrap();
            let point_configs = Self::build_point_configs(names, point_configs);
            if point_configs.is_empty() {
                Some(vec![])
            } else {
                Some(point_configs.iter().map(|conf| {
                    SubscriptionCriteria::new(conf.name.clone(), cot)
                }).collect())
            }
        } else if options.is_mapping() {
            let options = options.as_mapping().unwrap();
            let cot = options.get("cot").map(|v| serde_yaml::from_value(v.clone()).unwrap()).unwrap_or(Cot::All);
            let alarm = options.get("alarm").map(|v| v.as_u64().unwrap());
            let history = options.get("history").map(|v| serde_yaml::from_value(v.clone()).unwrap());
            trace!("{}.build_criterias | names: {:?}", self_id, names);
            let point_configs = Self::build_point_configs(names, point_configs);
            let creterias = point_configs
                .into_iter()
                .filter_map(|point_conf| {
                    Self::accept(self_id, &point_conf, &history, &alarm).then_some(SubscriptionCriteria::new(point_conf.name, cot))
                });
            if (creterias).clone().peekable().peek().is_some() {
                Some(creterias.collect())
            } else {
                None
            }
        } else {
            panic!("{}.build_criterias | Invalid subscribe options format: {:#?}", self_id, options);
        }
    }
    ///
    /// Returns all configs if names is empty, 
    /// otherwise returns configs for given names
    fn build_point_configs(names: &[serde_yaml::Value], configs: &[PointConfig]) -> Vec<PointConfig> {
        if names.is_empty() {
            configs.to_vec()
        } else {
            names.iter().filter_map(|name| {
                let name: String = serde_yaml::from_value(name.clone()).unwrap();
                configs.iter().find(|&point_conf| point_conf.name == name)
            }).cloned().collect()
        }
    }    
    ///
    /// Returns true if point_config is accepted by the options:
    ///     - alarm
    ///     - history
    fn accept(self_id: &str, point_conf: &PointConfig, history: &Option<PointConfigHistory>, alarm: &Option<u64>) -> bool {
        trace!("{}.accept | history: {:?}\t point.history: {:?}", self_id, history, point_conf.history);
        let mut accepted = true;
        if let Some(history) = history {
            trace!("{}.accept | check history", self_id);
            match history {
                PointConfigHistory::None => {}
                PointConfigHistory::Read => {
                    accepted &= point_conf.history == PointConfigHistory::Read
                }
                PointConfigHistory::Write => {
                    accepted &= point_conf.history == PointConfigHistory::Write
                }
                PointConfigHistory::ReadWrite => {
                    trace!("{}.accept | point_conf.history != PointConfigHistory::None: {}", self_id, point_conf.history != PointConfigHistory::None);
                    accepted &= point_conf.history != PointConfigHistory::None;
                }
            };
        }
        trace!("{}.accept | accepted: {}", self_id, accepted);
        if let Some(alarm) = alarm {
            trace!("{}.accept | check alarm", self_id);
            if *alarm == 0 {
                accepted &= point_conf.alarm.map(|point_alarm| {
                    point_alarm == 0
                }).unwrap_or(true);
            } else {
                accepted &= point_conf.alarm.map(|point_alarm| {
                    point_alarm >= (*alarm as u8)
                }).unwrap_or(false);
            }
        }
        trace!("{}.accept | accepted: {}", self_id, accepted);
        accepted
    }    
}