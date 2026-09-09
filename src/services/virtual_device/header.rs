use sal_core::dbg::Dbg;
use sal_sync::collections::FxIndexMap;

///
/// 
pub struct Header {
    block_row: u32,
    index_row: u32,
    unit_row: u32,
    input_block: String,
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
        let mut is_input = false;
        if !name.is_empty() {
            let mut cols = FxIndexMap::default();
            for col in column..(column + 3) {
                match index.get(col as usize) {
                    Some(index_val) => match index_val.as_str_opt() {
                        Some("name") | Some("time") | Some("value") => _ = {
                            cols.insert(index_val.as_string_opt().unwrap(), col);
                            is_input = true;
                        },
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
                    Some(HeaderBlock { name: name.to_owned(), cols, is_input })
                }
                ["result", "status", "target"] => {
                    Some(HeaderBlock { name: name.to_owned(), cols, is_input })
                }
                ["result", "target"] => {
                    Some(HeaderBlock { name: name.to_owned(), cols, is_input })
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
    pub fn from(parent: impl Into<String>, sheet: &spreadsheet_ods::Sheet) -> Self {
        let dbg = Dbg::new(parent, "Header");
        let (rows, columns) = sheet.used_grid_size();
        // log::debug!("{dbg}.from |    rows: {}", rows);
        // log::debug!("{dbg}.from | columns: {}", columns);
        let mut input_block = String::new();
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
                    if h_unit.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v.to_lowercase() == "unit").then(|| row_ix)).flatten() ).flatten() {
                            h_unit_row = ix;
                            h_unit = Some(row);
                            continue;
                        }
                    }
                    log::trace!("{dbg}.from | \n block: {:?},  \n index: {:?}", block, index);
                    for (column, block_name) in block.iter().enumerate() {
                        log::trace!("{dbg}.from | Column: {}, Block: {:?}", column, block_name);
                        if let Some(block_name) = block_name.as_str_opt() {
                            if let Some(block) = Self::parse_block(&dbg, block_name, column as u32, index) {
                                // log::debug!("{dbg}.from | Block: {:#?}", block);
                                if block.is_input {
                                    input_block = block.name.clone();
                                }
                                blocks.insert(block_name.to_owned(), block);
                            }
                        }
                    }
                    break;
                }
                _ => {
                    // log::debug!("{dbg}.from | row: {:?}", row);
                    if h_block.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v.to_lowercase() == "block").then(|| row_ix)).flatten() ).flatten() {
                            // log::debug!("{dbg}.from | Block row: {ix}");
                            h_block_row = ix;
                            h_block = Some(row);
                            continue;
                        }
                    }
                    if h_index.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v.to_lowercase() == "index").then(|| row_ix)).flatten() ).flatten() {
                            // log::debug!("{dbg}.from | Index row: {ix}");
                            h_index_row = ix;
                            h_index = Some(row);
                            continue;
                        }
                    }
                    if h_unit.is_none() {
                        if let Some(ix) = row.get(0).map(|v| v.as_str_opt().map(|v| (v.to_lowercase() == "unit").then(|| row_ix)).flatten() ).flatten() {
                            // log::debug!("{dbg}.from | Unit row: {ix}");
                            h_unit_row = ix;
                            h_unit = Some(row);
                        }
                    }
                }
            } 
        }
        if input_block.is_empty() {
            log::error!("{dbg}.from | Input block is not found");
        }
        Self {
            block_row: h_block_row,
            index_row: h_index_row,
            unit_row: h_unit_row,
            input_block,
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
    /// Returns Input [HeaderBlock]
    pub fn input_block(&self) -> Option<&HeaderBlock> {
        self.blocks.get(&self.input_block)
    }
    ///
    /// Returns row by index from `Sheet`
    fn row(sheet: &spreadsheet_ods::Sheet, row: u32, columns: u32) -> Vec<spreadsheet_ods::Value> {
        let mut out = vec![spreadsheet_ods::Value::Empty; columns as usize];
        for ((_row, col), cell) in sheet.iter_rows((row, 0)..(row + 1, columns)) {
            out[col as usize] = cell.value.to_owned();
        }
        out
        // sheet
        //     .iter_rows((row, 0)..(row + 1, columns))
        //     .map(|((_row, _col), cell)| cell.value.to_owned())
        //     .collect()
    }
}

///
/// 
#[derive(Debug)]
pub struct HeaderBlock {
    /// `true` when the block is input values
    is_input: bool,
    name: String,
    cols: FxIndexMap<String, u32>,
}
//
//
impl HeaderBlock {
    pub fn get(&self, key: &str) -> Option<u32> {
        self.cols.get(key).map(|v| *v)
    }
    ///
    /// Returns `true` when the block is input values
    pub fn is_input(&self) -> bool {
        self.is_input
    }
}
//
impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            // .field("block_row", &self.block_row)
            // .field("index_row", &self.index_row)
            // .field("unit_row", &self.unit_row)
            // .field("input_block", &self.input_block)
            .field("blocks", &self.blocks)
            // .field("dbg", &self.dbg)
            .finish()
    }
}