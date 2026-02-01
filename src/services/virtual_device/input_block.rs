use std::{fmt::Debug, time::Duration};

use sal_sync::services::entity::Point;

use crate::services::Header;

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
    time_col: u32,
    name_col: u32,
    value_col: u32,
    pub time: Duration,
    pub name: String,
    value: spreadsheet_ods::Value,
}
//
//
impl InputBlock {
    ///
    /// - `name` - the name of the block with input events in the table
    /// - `time_col` - name of the column event 'time'
    /// - `name_col` - name of the column event 'name'
    /// - `value_col` - name of the column event 'value'
    pub fn new(time_col: impl Into<String>, name_col: impl Into<String>, value_col: impl Into<String>, header: &Header) -> Self {
        let block = header.input_block().expect(&format!("Can't find Input block in the table"));
        let time_col = time_col.into();
        let name_col = name_col.into();
        let value_col = value_col.into();
        let time_col = block.get(&time_col).expect(&format!("Can't find '{}' column in the Input block of the table", time_col));
        let name_col = block.get(&name_col).expect(&format!("Can't find '{}' column in the Input block of the table", name_col));
        let value_col = block.get(&value_col).expect(&format!("Can't find '{}' column in the Input block of the table", value_col));
        Self {
            time_col,
            name_col,
            value_col,
            time: Default::default(),
            name: Default::default(),
            value: Default::default(),
        }
    }
    pub fn from_row(&self, row: &Vec<spreadsheet_ods::Value>) -> Option<Self> {
        let time = row.get(self.time_col as usize)?.as_u64_opt()?;
        let name = row.get(self.name_col as usize)?.as_string_opt()?;
        let value = row.get(self.value_col as usize)?.clone();
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
    pub fn to_point(&self, txid: usize) -> Point {
        match &self.value {
            spreadsheet_ods::Value::Empty => todo!(),
            spreadsheet_ods::Value::Boolean(v) => Point::new(txid, &self.name, *v),
            spreadsheet_ods::Value::Number(v) => Point::new(txid, &self.name, *v),
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