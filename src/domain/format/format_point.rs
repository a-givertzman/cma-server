use std::str::FromStr;

use regex::RegexBuilder;
use sal_core::error::Error;
use sal_sync::{collections::FxHashMap, services::entity::Point};
///
/// Сегмент скомпилированного текстового шаблона SQL.
#[derive(Debug, Clone)]
enum Token {
    Static(String),
    Dynamic { full_name: String, name: String, suffix: Sufix },
}
///
/// Варианты суфикса маркера
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sufix {
    Name,
    Value,
    Ts,
    Status,
    None,
}
impl FromStr for Sufix {
    type Err = Error;
    fn from_str(sufix: &str) -> Result<Self, Self::Err> {
        match sufix {
            "name" => Ok(Self::Name),
            "value" => Ok(Self::Value),
            "timestamp" => Ok(Self::Ts),
            "status" => Ok(Self::Status),
            _ => Err(Error::new("FormatPoint.Sufix", "from_str").err(format!("Unknown sufix '{sufix}', expected name/value/timestamp/status")))
        }
    }
}
///
/// ### Шаблонизатор для подстановки параметров `Point` в строку
/// 
/// **Виды поддерживаемых маркеров для `Point`:**
/// ```ignore
///     - {input}  - по умолчанию {input.value}
///     - {input.name}
///     - {input.value}
///     - {input.timestamp}
///     - {input.status}
/// ```
///
/// **Example 1**
/// ```ignore
/// Point {
///     name: "test-point",
///     value: 12,
///     timestamp: "",
///     status: Ok,
/// }
/// "select * from table where id = {point.name}"      => "select * from table where id = test-point"
/// "select * from table where id = {point.value}"     => "select * from table where id = 12"
/// "select * from table where id = {point.timestamp}" => "select * from table where id = "
/// "select * from table where id = {point.status}"    => "select * from table where id = 0"
/// ```
///
/// **Example 2**
/// - formating string: `insert into temperature (id, value) values ({input1.status}, {input1.value});`
/// - values can be added using insert method `format.insert("input1", point)`
/// - values: `point.status = 1; point.value = 19,7`
/// - out   : `"insert into temperature (id, value) values (0, 19,7)"`
pub struct FormatPoint {
    tokens: Vec<Token>,
    names: FxHashMap<String, (String, Sufix)>,
    values: FxHashMap<String, Point>,
}
// 
impl FormatPoint {
    ///
    /// Creates new instance of the Format from configuration string
    pub fn new(input: &str) -> Result<Self, Error> {
        let re = r#"\{(.*?)\}"#;
        let re = RegexBuilder::new(re).multi_line(true).build().unwrap();
        let mut tokens = Vec::new();
        let mut names = FxHashMap::default();
        let mut last_idx = 0;
        for cap in re.captures_iter(input) {
            let mat = cap.get(0).unwrap();
            if mat.start() > last_idx {
                tokens.push(Token::Static(input[last_idx..mat.start()].to_string()));
            }
            let full_name = cap.get(1).unwrap().as_str().to_string();
            let mut parts = full_name.split('.').map(|part| part.into());
            let name: String = parts.next().unwrap();
            let suffix = match parts.next() {
                Some(sufix) => Sufix::from_str(&sufix).map_err(|err| Error::new("FormatPoint", "new").pass(err))?,
                None => Sufix::None,
            };
            names.insert(full_name.clone(), (name.clone(), suffix.clone()));
            tokens.push(Token::Dynamic { full_name, name, suffix });
            last_idx = mat.end();
        }
        if last_idx < input.len() {
            tokens.push(Token::Static(input[last_idx..].to_string()));
        }
        Ok(Self { tokens, names, values: FxHashMap::default() })
    }
    ///
    /// Вносит актуальное значение `Point` для подстановки в шаблон.
    /// - `key`: Полное имя маркера (например, "input1.value")
    pub fn insert(&mut self, key: &str, value: Point) {
        self.values.insert(key.into(), value);
    }
    ///
    /// Returns formatted string? replacing configured markers with the associated values by them keys
    pub fn out(&self) -> String {
        let mut out = String::with_capacity(256);
        for token in &self.tokens {
            match token {
                Token::Static(s) => out.push_str(s),
                Token::Dynamic { full_name, name, suffix } => {
                    if let Some(point) = self.values.get(full_name) {
                        let value = match suffix {
                            Sufix::Name => point.name(),
                            Sufix::Value | Sufix::None => point.value().to_string(),
                            Sufix::Ts => point.timestamp().to_string(),
                            Sufix::Status => point.status().to_string(),
                        };
                        out.push_str(&value);
                    } else {
                        out.push_str("{");
                        out.push_str(full_name);
                        out.push_str("}");
                    }
                }
            }
        }
        out
    }
    ///
    /// Returns List of al names & sufixes in the following format:
    /// ```ignore
    /// Map<fullName, (name, sufix)>
    /// 
    /// - Keep in maind, the name can be:
    ///      input | sufix      |
    ///      name  |            |
    ///     - input  - by defoult input.value will be used
    ///     - input.name
    ///     - input.value
    ///     - input.timestamp
    ///     - input.status
    /// ```
    pub fn markers(&self) -> FxHashMap<String, (String, Sufix)> {
        self.names.clone()
    }
    #[deprecated(since = "0.3.1", note = "Пожалуйста, удалите метод, он ничего не делает")]
    pub fn prepare(&mut self) {}
}
//
impl std::fmt::Display for FormatPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out())
    }
}
//
impl std::fmt::Debug for FormatPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.out())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use sal_sync::services::entity::{Point, PointHlr, Status, Cot};
    use chrono::Utc;
    /// Вспомогательный метод для штамповки тестовых точек Point::Int
    fn create_test_point(name: &str, value: i64, status: Status) -> Point {
        Point::Int(PointHlr::new(
            0,
            name,
            value,
            status,
            Cot::Inf,
            Utc::now(),
        ))
    }
    #[test]
    fn test_format_all_valid_suffixes() {
        let template = "INSERT INTO metrics (tag, val, stat, ts) VALUES ('{input.name}', {input.value}, {input.status}, '{input.timestamp}');";
        let mut formatter = FormatPoint::new(template).unwrap();
        let point = create_test_point("Sensor_A", 105, Status::Ok);
        formatter.insert("input.name", point.clone());
        formatter.insert("input.value", point.clone());
        formatter.insert("input.status", point.clone());
        formatter.insert("input.timestamp", point.clone());
        let result = formatter.out();
        assert!(result.contains("'Sensor_A'"));
        assert!(result.contains("105"));
        assert!(result.contains("0"));
    }
    #[test]
    fn test_format_default_suffix_fallback() {
        let template = "SELECT * FROM table WHERE value = {input};";
        let mut formatter = FormatPoint::new(template).unwrap();
        let point = create_test_point("Sensor_B", 42, Status::Ok);
        formatter.insert("input", point);
        assert_eq!(formatter.out(), "SELECT * FROM table WHERE value = 42;");
    }
    #[test]
    fn test_format_missing_value_leaves_placeholder() {
        let template = "SELECT {missing.value} FROM table WHERE id = {present.value};";
        let mut formatter = FormatPoint::new(template).unwrap();
        let point = create_test_point("Sensor_C", 77, Status::Ok);
        formatter.insert("present.value", point);
        assert_eq!(formatter.out(), "SELECT {missing.value} FROM table WHERE id = 77;");
    }
    #[test]
    fn test_format_no_placeholders() {
        let raw_sql = "SELECT * FROM static_table WHERE id = 10;";
        let formatter = FormatPoint::new(raw_sql).unwrap();
        assert_eq!(formatter.out(), raw_sql);
    }
    #[test]
    fn test_format_empty_template() {
        let formatter = FormatPoint::new("").unwrap();
        assert_eq!(formatter.out(), "");
    }
    #[test]
    fn test_format_multoline_template() {
        let template = "UPDATE table\nSET val = {input.value}\nWHERE name = '{input.name}';";
        let mut formatter = FormatPoint::new(template).unwrap();
        let point = create_test_point("Sensor_D", 99, Status::Ok);
        formatter.insert("input.value", point.clone());
        formatter.insert("input.name", point);
        let expected = "UPDATE table\nSET val = 99\nWHERE name = 'Sensor_D';";
        assert_eq!(formatter.out(), expected);
    }
    #[test]
    #[should_panic(expected = "Unknown suffix in tag")]
    fn test_format_unknown_suffix_panics() {
        let template = "SELECT * FROM table WHERE val = {input.corrupted_property};";
        let mut formatter = FormatPoint::new(template).unwrap();
        let point = create_test_point("Sensor_E", 100, Status::Ok);
        formatter.insert("input.corrupted_property", point);
        let _ = formatter.out();
    }
    #[test]
    fn test_format_display_and_debug_traits() {
        let template = "VALUES ({input.value})";
        let mut formatter = FormatPoint::new(template).unwrap();
        let point = create_test_point("Sensor_F", 12, Status::Ok);
        formatter.insert("input.value", point);
        let display_output = format!("{}", formatter);
        let debug_output = format!("{:?}", formatter);
        assert_eq!(display_output, "VALUES (12)");
        assert_eq!(debug_output, "VALUES (12)");
    }
    #[test]
    fn test_format_names_extraction() {
        let template = "SELECT * FROM {table} WHERE val = {input.value} AND status = {input.status};";
        let formatter = FormatPoint::new(template).unwrap();
        let names = formatter.markers();
        assert_eq!(names.len(), 3);
        assert!(names.contains_key("table"));
        assert!(names.contains_key("input.value"));
        assert!(names.contains_key("input.status"));
        let (name, suffix) = names.get("input.value").unwrap();
        assert_eq!(name, "input");
        assert_eq!(*suffix, Sufix::Value);
    }
}
