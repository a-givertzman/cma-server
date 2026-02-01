use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;

///
/// 
pub struct Header {
    block_row: u32,
    index_row: u32,
    unit_row: u32,
    blocks: FxIndexMap<String, HeaderBlock>,
    dbg: Dbg,
}
//
impl Header {
    ///
    /// ## Parse [HeaderBlock] from:
    /// 
    ///  - **`Input values`** blok
    /// ```
    /// | ------------------- |
    /// | time | name | value |
    /// | ------------------- |
    /// ```
    /// 
    ///  - **`Result & Target`** blok
    /// ```
    /// | ---------------------------- |
    /// | target     | result | status |
    /// | ---------------------------- |
    fn parse_block(dbg: &Dbg, name: &str, column: u32, index: &Vec<spreadsheet_ods::Value>) -> Option<HeaderBlock> {
        if !name.is_empty() {
            let mut cols = FxIndexMap::default();
            for col in column..(column + 3) {
                match index.get(col as usize) {
                    Some(index_val) => match index_val.as_str_opt() {
                        Some("name") | Some("time") | Some("value") => _ = cols.insert(index_val.as_string_opt().unwrap(), col),
                        Some("target") | Some("result") | Some("status") => _ = cols.insert(index_val.as_string_opt().unwrap(), col),
                        _ => {},
                    }
                    None => log::debug!("{dbg}.from | col {col} - out of bounds"),
                };
            }
            let mut keys: Vec<&str> = cols.keys().map(|v| v.as_str()).collect();
            keys.sort();
            match keys.as_slice() {
                ["name", "time", "value"] => {
                    Some(HeaderBlock { name: name.to_owned(), cols })
                }
                ["result", "status", "target"] => {
                    Some(HeaderBlock { name: name.to_owned(), cols })
                }
                ["result", "target"] => {
                    Some(HeaderBlock { name: name.to_owned(), cols })
                }
                _ => None
            }
        } else {
            None
        }
    } 
    ///
    /// Retirns [Header] parsed from `Sheet`
    /// 
    /// Header has folowing structure: 
    /// ```
    /// |   Input values      |     Result & Target          | Result & Target | ...
    /// | ------------------- | ---------------------------- |       ...       |...
    /// | time | name | value | TargetName |  -     |  -     |       ...       |...
    /// | ------------------- | ---------------------------- |       ...       |...
    /// | ms   |  -   |  -    | target     | result | status |       ...       |...
    /// | ------------------- | ---------------------------- |       ...       |...
    /// ```
    pub fn from(sheet: &spreadsheet_ods::Sheet) -> Self {
        let dbg = Dbg::own("Header");
        let rows = sheet.row_header_max().min(10);
        let columns = sheet.col_header_max();
        log::debug!("{dbg}.from |    rows: {}", rows);
        log::debug!("{dbg}.from | columns: {}", columns);
        let mut h_block_row = 0;
        let mut h_index_row = 0;
        let mut h_unit_row = 0;
        let mut h_block: Option<Vec<spreadsheet_ods::Value>> = None;
        let mut h_index: Option<Vec<spreadsheet_ods::Value>> = None;
        let mut h_unit: Option<Vec<spreadsheet_ods::Value>> = None;
        let mut blocks = FxIndexMap::default();
        for row_ix in 0..rows {
            let row = Self::row(&sheet, row_ix, columns);
            match (&h_block, &h_index) {
                (Some(block), Some(index)) => {
                    log::debug!("{dbg}.from | \n block: {:?},  \n index: {:?}", block, index);
                    for (column, block_name) in block.iter().enumerate() {
                        log::debug!("{dbg}.from | Column: {}, Block: {:?}", column, block_name);
                        if let Some(block_name) = block_name.as_str_opt() {
                            if let Some(block) = Self::parse_block(&dbg, block_name, column as u32, index) {
                                log::debug!("{dbg}.from | Block: {:#?}", block);
                                blocks.insert(block_name.to_owned(), block);
                            }
                        }
                    }
                    break;
                }
                _ => {
                    if h_block.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v == "block").then(|| row_ix)).flatten() ).flatten() {
                            log::debug!("{dbg}.from | Block row: {ix}");
                            h_block_row = ix;
                            h_block = Some(row);
                            continue;
                        }
                    }
                    if h_index.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v == "index").then(|| row_ix)).flatten() ).flatten() {
                            log::debug!("{dbg}.from | Index row: {ix}");
                            h_index_row = ix;
                            h_index = Some(row);
                            continue;
                        }
                    }
                    if h_unit.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v == "unit").then(|| row_ix)).flatten() ).flatten() {
                            log::debug!("{dbg}.from | Unit row: {ix}");
                            h_unit_row = ix;
                            h_unit = Some(row);
                        }
                    }
                }
            } 
        }
        Self {
            block_row: h_block_row,
            index_row: h_index_row,
            unit_row: h_unit_row,
            blocks,
            dbg,
        }
    }
    ///
    /// Returns bottom row of the [Header]
    pub fn end(&self) -> u32 {
        self.unit_row
    }
    ///
    /// Returns [HeaderBlock] by it's name
    pub fn block(&self, key: &str) -> Option<&HeaderBlock> {
        self.blocks.get(key)
    }
    ///
    /// Returns row by index from `Sheet`
    fn row(sheet: &spreadsheet_ods::Sheet, row: u32, columns: u32) -> Vec<spreadsheet_ods::Value> {
        sheet
            .iter_rows((row, 0)..(row + 1, columns))
            .map(|((_row, _col), cell)| cell.value.to_owned())
            .collect()
    }
}

///
/// 
#[derive(Debug)]
pub struct HeaderBlock {
    name: String,
    cols: FxIndexMap<String, u32>,
}
//
//
impl HeaderBlock {
    pub fn get(&self, key: &str) -> Option<u32> {
        self.cols.get(key).map(|v| *v)
    }
}