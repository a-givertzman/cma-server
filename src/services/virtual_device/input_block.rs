use std::{fmt::Debug, time::Duration};

use sal_sync::services::entity::Point;

///
/// Parses input values from the block of the tabe
/// ```
/// |   Input values      |
/// | ------------------- |
/// | time | name | value |
/// | ------------------- |
/// | ms   |  -   |  -    |
/// | ------------------- |
/// ```
// #[derive(Debug)]
pub struct InputBlock {
    time_col: usize,
    name_col: usize,
    value_col: usize,
    pub time: Duration,
    pub name: String,
    value: spreadsheet_ods::Value,
}
//
//
impl InputBlock {
    pub fn new(time_col: usize, name_col: usize, value_col: usize) -> Self {
        Self {
            time_col,
            name_col,
            value_col,
            time: Default::default(),
            name: Default::default(),
            value: spreadsheet_ods::Value::Empty,
        }
    }
    pub fn from_row(&self, row: &Vec<spreadsheet_ods::Value>) -> Option<Self> {
        let time = row.get(self.time_col)?.as_u64_opt()?;
        let name = row.get(self.name_col)?.as_string_opt()?;
        let value = row.get(self.value_col)?.clone();
        Some(Self {
            time_col: self.time_col,
            name_col: self.name_col,
            value_col: self.value_col,
            time: Duration::from_millis(time as u64),
            name,
            value,
        })
    }
    ///
    /// 
    pub fn to_point(&self) -> Point {
        match &self.value {
            spreadsheet_ods::Value::Empty => todo!(),
            spreadsheet_ods::Value::Boolean(v) => Point::new(0, &self.name, *v),
            spreadsheet_ods::Value::Number(v) => Point::new(0, &self.name, *v),
            spreadsheet_ods::Value::Percentage(_) => todo!(),
            spreadsheet_ods::Value::Currency(_, _) => todo!(),
            spreadsheet_ods::Value::Text(_) => todo!(),
            spreadsheet_ods::Value::TextXml(_) => todo!(),
            spreadsheet_ods::Value::DateTime(_) => todo!(),
            spreadsheet_ods::Value::TimeDuration(_) => todo!(),
        }
    }
}
//
//
impl Debug for InputBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputBlock")
            // .field("time_col", &self.time_col)
            // .field("name_col", &self.name_col)
            // .field("value_col", &self.value_col)
            .field("time", &self.time)
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}