use sal_core::{dbg::Dbg, error::Error};
use spreadsheet_ods::{Sheet, Value, WorkBook};

///
/// Contains `WorkBoock`
/// - provides basic functionality and access to the `Sheet`'s
pub struct Table {
    path: String,
    book: WorkBook,
    sheet: usize,
    dbg: Dbg,
}
//
//
impl Table {
    ///
    /// Returns [Book] new instance read from `path`
    /// - `path` - path to ODS file
    /// - `sheet` - name of the sheet to be set as active, later active Sheet can be changed by calling `Table::change_sheet(name)`
    pub fn load(parent: impl Into<String>, path: impl Into<String>, sheet: impl Into<String>) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, "Table");
        let error = Error::new(&dbg, "load");
        let path = path.into();
        let sheet = sheet.into();
        let book = spreadsheet_ods::read_ods(&path)
            .map_err(|err| error.pass_with(format!("Can't read table from '{path}'"), err.to_string()))?;
        log::debug!("{dbg}.run | Table loaded: '{path}'");
        let sheet = book.sheet_idx(&sheet)
            .ok_or(error.err(format!("Worksheet '{sheet}' - not found")))?;
        log::debug!("{dbg}.run | Active sheet: '{}' [{sheet}]", book.sheet(sheet).name());
        Ok(Self {
            path,
            book,
            sheet,
            dbg,
        })
    }
    ///
    /// Stores table into file in ODS format
    pub fn store(&mut self) -> Result<(), Error> {
        let error = Error::new(&self.dbg, "store");
        spreadsheet_ods::write_ods(&mut self.book, &self.path)
            .map_err(|err| error.pass_with(format!("Can't read table from '{}'", self.path), err.to_string()))
    }
    ///
    /// Returns active `Sheet`
    pub fn sheet(&self) -> &Sheet {
        self.book.sheet(self.sheet)
    }
    ///
    /// Returns number of rows of active sheet
    pub fn rows(&self) -> u32 {
        let (rows, _) = self.book.sheet(self.sheet).used_grid_size();
        rows
    }
    ///
    /// Returns number of columns of active sheet
    pub fn columns(&self) -> u32 {
        let (_, cols) = self.book.sheet(self.sheet).used_grid_size();
        cols
    }
    ///
    /// Returns active `Sheet` mutable
    pub fn sheet_mut(&mut self) -> &mut Sheet {
        self.book.sheet_mut(self.sheet)
    }
    ///
    /// Returns row by index from active `Sheet` mutable
    pub fn row(&self, i: u32, cols: u32) -> Vec<Value> {
        self.book.sheet(self.sheet)
            .iter_rows((i, 0)..(i + 1, cols))
            .map(|((_row, _col), cell)| cell.value.to_owned())
            .collect()
    }
}