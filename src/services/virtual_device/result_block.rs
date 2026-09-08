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
    pub fn new(name: impl Into<String>, target_col: impl AsRef<str>, result_col: impl AsRef<str>, status_col: impl AsRef<str>, header: &Header) -> Self {
        let name = name.into();
        let block = header.block(&name).expect(&format!("Can't find '{name}' block in the table"));
        let target_col = block.get(target_col.as_ref()).unwrap_or_else(|| panic!("Can't find '{}' column in the '{name}' block of the table", target_col.as_ref()));
        let result_col = block.get(&result_col.as_ref()).unwrap_or_else(|| panic!("Can't find '{}' column in the '{name}' block of the table", result_col.as_ref()));
        let status_col = block.get(&status_col.as_ref()).unwrap_or_else(|| panic!("Can't find '{}' column in the '{name}' block of the table", status_col.as_ref()));
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
    /// Returns values of the result block from the specified table
    pub fn from_table(self, t: &Table, row: u32) -> Option<Self> {
        let target = t.sheet().value(row, self.target_col).clone();
        let result = t.sheet().value(row, self.result_col).clone();
        let status = t.sheet().value(row, self.status_col).clone();
        Some(Self {
            name: self.name,
            target_col: self.target_col,
            result_col: self.result_col,
            status_col: self.status_col,
            target,
            result,
            status,
        })
    }
    ///
    /// Returns values of the result block from the specified row
    pub fn write_result(&self, row_ix: u32, value: impl Into<spreadsheet_ods::Value>, table: &mut Table) {
        // let value = spreadsheet_ods::Value::Number(result);
        table.sheet_mut().set_value(row_ix, self.result_col, value);
    }
    /// Returns `true` if result block contains target value
    pub fn has_target(&self) -> bool {
        log::debug!("{}.has_target | '{}' | target: {:?},    type: {:?}", crate::me::<Self>(), self.name, self.target, self.target.value_type());
        self.target.value_type() != spreadsheet_ods::ValueType::Empty
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