///
/// ConfTree holds sede_yaml::Value and it key
/// for root key = ""
/// Allow to iterate across all yaml config nodes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfTree {
    pub key: String,
    pub conf: serde_yaml::Value,
}
//
// 
impl ConfTree {
    ///
    /// creates iterotor on the serde_yaml::Value mapping
    pub fn new_root(conf: serde_yaml::Value) -> Self {
        Self {
            key: String::new(),
            conf,
        }
    }
    ///
    /// creates ConfTree instance holding the key and serde_yaml::Value
    pub fn new(key: &str, conf: serde_yaml::Value) -> Self {
        Self {key: key.into(), conf}
    }
    ///
    /// returns true if holding mapping 
    pub fn is_mapping(&self) -> bool {
        self.conf.is_mapping()
    }
    ///
    /// iterates across all sub nodes 
    pub fn next(&self) -> Option<ConfTree> {
        match self.sub_nodes() {
            Some(mut sub_nodes) => sub_nodes.next(),
            None => None,
        }
    }
    ///
    /// returns count of sub nodes
    pub fn count(&self) -> usize {
        match self.sub_nodes() {
            Some(sub_nodes) => sub_nodes.count(),
            None => 0,
        }
    }
    ///
    /// iterate across all sub nodes
    pub fn sub_nodes(&self) -> Option<impl Iterator<Item = ConfTree> + '_> {
        if self.conf.is_mapping() {
            let iter = self.conf.as_mapping().unwrap().into_iter().map( |(key, value)| {
                ConfTree::new(
                    key.as_str().unwrap(),
                    value.clone(),
                )
            });
            Some(
                iter
            )
        } else {
            None
        }
    }
    ///
    /// returns tree node by it's key if exists
    pub fn get(&self, key: &str) -> Option<ConfTree> {
        if self.conf.is_mapping() {
            self.conf.as_mapping().unwrap().get(key).map(|value| ConfTree {
                key: key.to_string(),
                conf: value.clone(),
            })
        } else {
            None
        }
    }
    ///
    /// returns tree node value as bool by it's key if exists
    pub fn as_bool(&self, key: &str) -> Result<bool, String> {
        if self.conf.is_mapping() {
            match self.conf.as_mapping().unwrap().get(key) {
                Some(value) => {
                    match value.as_bool() {
                        Some(value) => Ok(value),
                        None => Err(format!("error getting BOOL by key '{:?}' from node '{:?}'", &key, value)),
                    }
                }
                None => Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf)),
            }
        } else {
            Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf))
        }
    }
    ///
    /// returns tree node value as bool by it's key if exists
    pub fn as_i64(&self, key: &str) -> Result<i64, String> {
        if self.conf.is_mapping() {
            match self.conf.as_mapping().unwrap().get(key) {
                Some(value) => {
                    match value.as_i64() {
                        Some(value) => Ok(value),
                        None => Err(format!("error getting INT by key '{:?}' from node '{:?}'", &key, value)),
                    }
                }
                None => Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf)),
            }
        } else {
            Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf))
        }
    }
    ///
    /// returns tree node value as f32 by it's key if exists
    pub fn as_f32(&self, key: &str) -> Result<f32, String> {
        if self.conf.is_mapping() {
            match self.conf.as_mapping().unwrap().get(key) {
                Some(value) => {
                    match value.as_f64() {
                        Some(value) => Ok(value as f32),
                        None => Err(format!("error getting REAL by key '{:?}' from node '{:?}'", &key, value)),
                    }
                }
                None => Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf)),
            }
        } else {
            Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf))
        }
    }
    ///
    /// returns tree node value as f64 by it's key if exists
    pub fn as_f64(&self, key: &str) -> Result<f64, String> {
        if self.conf.is_mapping() {
            match self.conf.as_mapping().unwrap().get(key) {
                Some(value) => {
                    match value.as_f64() {
                        Some(value) => Ok(value),
                        None => Err(format!("error getting DOUBLE by key '{:?}' from node '{:?}'", &key, value)),
                    }
                }
                None => Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf)),
            }
        } else {
            Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf))
        }
    }
    ///
    /// returns tree node value as str by it's key if exists
    pub fn as_str(&self, key: &str) -> Result<&str, String> {
        if self.conf.is_mapping() {
            match self.conf.as_mapping().unwrap().get(key) {
                Some(value) => {
                    match value.as_str() {
                        Some(value) => Ok(value),
                        None => Err(format!("Error getting STRING by key '{:?}' from node '{:?}'", &key, value)),
                    }
                }
                None => Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf)),
            }
        } else {
            Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf))
        }
    }
    ///
    /// removes node by it's key if exists
    /// returns Result<&Self>
    pub fn remove(&mut self, key: &str) -> Result<serde_yaml::Value, String> {
        if self.conf.is_mapping() {
            match self.conf.as_mapping_mut().unwrap().remove(key) {
                Some(value) => Ok(value),
                None => Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf)),
            }
        } else {
            Err(format!("Key '{:?}' not found in the node '{:?}'", &key, &self.conf))
        }
    }
}
