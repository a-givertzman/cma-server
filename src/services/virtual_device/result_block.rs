use std::fmt::Debug;

use crate::services::Header;

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
    target_col: (String, Option<usize>),
    result_col: (String, Option<usize>),
    status_col: (String, Option<usize>),
    target: spreadsheet_ods::Value,
    result: spreadsheet_ods::Value,
    status: spreadsheet_ods::Value,
}
//
//
impl ResultBlock {
    pub fn new(name: impl Into<String>, target_col: impl Into<String>, result_col: impl Into<String>, ctatus_col: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            target_col: (target_col.into(), None),
            result_col: (result_col.into(), None),
            status_col: (ctatus_col.into(), None),
            target: Default::default(),
            result: Default::default(),
            status: Default::default(),
        }
    }
    pub fn from_row(&self, header: &Header, row: &Vec<spreadsheet_ods::Value>) -> Option<Self> {
        let block = header.block(&self.name).unwrap();
        let target_col = block.get(&self.target_col.0).unwrap();
        let result_col = block.get(&self.result_col.0).unwrap();
        let status_col = block.get(&self.status_col.0).unwrap();
        let target = row.get(target_col).unwrap().to_owned();
        let result = row.get(result_col).unwrap().to_owned();
        let status = row.get(status_col).unwrap().to_owned();
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