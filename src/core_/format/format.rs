use std::collections::HashMap;
use log::trace;
use regex::RegexBuilder;
use crate::core_::point::point::Point;
///
/// Replaces input markers {marker name} with the concrete values
///
/// input marker can be:
/// ````
///      input | sufix      |
///      name  |            |
///     - input  - by defoult input.value will be used
///     - input.name
///     - input.value
///     - input.timestamp
///     - input.status
/// ````
/// - formating string: "insert into {table} (id, value) values ({input1.status}, {input1.value})"
/// - values can be added using insert method format.insert("input1", point)
/// - values: table = "temperature"; point.status = 1; point.value = 19,7
/// - out   : "insert into temperature (id, value) values (0, 19,7)"
pub struct Format {
    input: String,
    names: HashMap<String, (String, Option<String>)>,
    values: HashMap<String, Point>,
}
//
// 
impl Format {
    ///
    /// Creates new instance of the Format from configuration string
    pub fn new(input: &str) -> Self {
        let re = r#"\{(.*?)\}"#;
        let re = RegexBuilder::new(re).multi_line(true).build().unwrap();
        let names = re.captures_iter(input).map(|cap| {
            let full_name = cap.get(1).unwrap().as_str().to_string();
            let mut parts = full_name.split('.').map(|part| part.into());
            let name = parts.next().unwrap();
            let sufix = parts.next();
            (full_name, (name, sufix))
        }).collect();        
        trace!("Format.new | names {:?}", &names);
        Self {
            input: input.into(),
            names,
            values: HashMap::new(),
        }
    }
    ///
    /// Inserts a Point by key to the configured format
    pub fn insert(&mut self, key: &str, value: Point) {
        self.values.insert(key.into(), value);
    }
    ///
    /// Returns formatted string? replacing configured markers with the associated values by them keys
    pub fn out(&self) -> String {
        let mut input = self.input.clone();
        for (full_name, (name, sufix)) in &self.names {
            trace!("Format.out | fullName {:?}", full_name);
            if let Some(point) = self.values.get(full_name) {
                let value = match sufix {
                    Some(sufix) => {
                        match sufix.as_str() {
                            "name" => point.name(),
                            "value" => point.value().to_string(),
                            "timestamp" => point.timestamp().to_string(),
                            "status" => point.status().to_string(),
                            _ => panic!("Format.out | Unknown input sufix in: {:?}, allowed: .name / .value / .timestamp", &name),
                        }
                    }
                    None => {
                        trace!("Format.out | name: {:?}, sufix: None, taking point.value by default", &name);
                        point.value().to_string()
                    }
                };
                let pattern = format!("{{{}}}", full_name);
                trace!("Format.out | replacing pattern {:?} with value: {:?}", pattern, value);
                input = input.replace(&pattern, &value);
                trace!("Format.out | result: {:?}", input);
            };
        };
        input
    }
    ///
    /// Returns List of al names & sufixes in the following format:
    /// ```
    /// HashMap<fullName, (name, sufix)>
    /// ```
    /// - Keep in maind, the name can be:
    /// ````
    ///      input | sufix      |
    ///      name  |            |
    ///     - input  - by defoult input.value will be used
    ///     - input.name
    ///     - input.value
    ///     - input.timestamp
    ///     - input.status
    /// ````
    pub fn names(&self) -> HashMap<String, (String, Option<String>)> {
        self.names.clone()
    }
    ///
    /// Already inserted values will be stored into out, 
    /// and will be removed from the names. 
    /// Less number of remained values, faster the replacement
    pub fn prepare(&mut self) {
        let input = self.out();
        self.input = input;
        let values = self.values.clone();
        let names = values.keys();
        for name in names {
            self.names.remove(name);
            self.values.remove(name);
        };
        trace!("Format.prepare | self.input {:?}", self.input);
    }
}
//
//
impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out())
    }
}
//
// 
impl std::fmt::Debug for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out())
        // f.debug_struct("Format").field("input", &self.input).field("values", &self.values).finish()
    }
}