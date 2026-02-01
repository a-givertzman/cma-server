use std::fmt::Debug;

use crate::services::{Header, Table};

///
/// `Result & Target` block values parsed from the tabe block
/// ```
/// | ---------------------------- |
/// | TargetName |  -     |  -     |
/// | ---------------------------- |
/// | target     | result | status |
/// | ---------------------------- |
/// ```
// #[derive(Debug)]
pub struct ResultBlock {
    name: String,
    target_col: u32,
    result_col: u32,
    status_col: u32,
    target: spreadsheet_ods::Value,
    result: spreadsheet_ods::Value,
    status: spreadsheet_ods::Value,
}
//
//
impl ResultBlock {
    pub fn new(name: impl Into<String>, target_col: impl Into<String>, result_col: impl Into<String>, status_col: impl Into<String>, header: &Header) -> Self {
        let name = name.into();
        let block = header.block(&name).expect(&format!("Can't find '{name}' block in the table"));
        let target_col = target_col.into();
        let result_col = result_col.into();
        let status_col = status_col.into();
        let target_col = block.get(&target_col).expect(&format!("Can't find '{}' column in the '{name}' block of the table", target_col));
        let result_col = block.get(&result_col).expect(&format!("Can't find '{}' column in the '{name}' block of the table", result_col));
        let status_col = block.get(&status_col).expect(&format!("Can't find '{}' column in the '{name}' block of the table", status_col));
        Self {
            name: name.into(),
            target_col,
            result_col,
            status_col,
            target: Default::default(),
            result: Default::default(),
            status: Default::default(),
        }
    }
    ///
    /// Returns values of the result block from the specified row
    pub fn from_row(&self, row: &Vec<spreadsheet_ods::Value>) -> Option<Self> {
        let target = row.get(self.target_col as usize)?.to_owned();
        let result = row.get(self.result_col as usize)?.to_owned();
        let status = row.get(self.status_col as usize)?.to_owned();
        Some(Self {
            name: self.name.clone(),
            target_col: self.target_col.clone(),
            result_col: self.result_col.clone(),
            status_col: self.status_col.clone(),
            target,
            result,
            status,
        })
    }
    ///
    /// Returns values of the result block from the specified row
    pub fn write(&self, row_ix: u32, result: f64, table: &mut Table) {
        let value = spreadsheet_ods::Value::Number(result);
        table.sheet_mut().set_value(row_ix, self.target_col, value);
    }
}
//
//
impl Debug for ResultBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultBlock")
            .field("name", &self.name)
            // .field("target_col", &self.target_col)
            // .field("result_col", &self.result_col)
            // .field("status_col", &self.status_col)
            .field("target", &self.target)
            .field("result", &self.result)
            .field("status", &self.status)
            .finish()
    }
}