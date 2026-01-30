use std::{fmt::Debug, time::Duration};

use sal_sync::services::entity::Point;

///
/// 
// #[derive(Debug)]
pub struct InputEvent {
    time_col: usize,
    name_col: usize,
    value_col: usize,
    time: Duration,
    name: String,
    value: Point,
}
//
//
impl InputEvent {
    pub fn new(time_col: usize, name_col: usize, value_col: usize) -> Self {
        Self {
            time_col,
            name_col,
            value_col,
            time: Default::default(),
            name: Default::default(),
            value: Point::new(0, "Not sampled", 0),
        }
    }
    pub fn from_row(&self, row: &Vec<(u32, spreadsheet_ods::Value)>) -> Self {
        let time = row.get(self.time_col).unwrap().1.as_u64_or_default();
        let name = row.get(self.name_col).unwrap().1.as_str_or("Not found");
        let value = match &row.get(self.value_col).unwrap().1 {
            spreadsheet_ods::Value::Empty => todo!(),
            spreadsheet_ods::Value::Boolean(v) => Point::new(0, &name, *v),
            spreadsheet_ods::Value::Number(v) => Point::new(0, &name, *v),
            spreadsheet_ods::Value::Percentage(_) => todo!(),
            spreadsheet_ods::Value::Currency(_, _) => todo!(),
            spreadsheet_ods::Value::Text(_) => todo!(),
            spreadsheet_ods::Value::TextXml(_) => todo!(),
            spreadsheet_ods::Value::DateTime(_) => todo!(),
            spreadsheet_ods::Value::TimeDuration(_) => todo!(),
        };
        Self {
            time_col: self.time_col,
            name_col: self.name_col,
            value_col: self.value_col,
            time: Duration::from_millis(time as u64),
            name: name.to_owned(),
            value,
        }
    }
}
//
//
impl Debug for InputEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputEvent")
            // .field("time_col", &self.time_col)
            // .field("name_col", &self.name_col)
            // .field("value_col", &self.value_col)
            .field("time", &self.time)
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}