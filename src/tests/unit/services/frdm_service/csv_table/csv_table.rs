use std::{fmt::Debug, fs::OpenOptions, path::Path, str::FromStr};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{domain::{FxIndexMap, FxIndexMapExt}, err, err_pass, tests::unit::services::frdm_service::CsvRow};

type Header = Vec<CsvField>;
type Record = Vec<String>;

/// ### Поле таблицы `CsvTable`
pub struct CsvField {
    /// Строковой уникальный идентификатор столбца
    pub key: String,
    /// Порядковый номер столбца в таблице
    pub ix: usize,
}
impl Debug for CsvField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.key, self.ix)
    }
}
/// ### Хранит данные, распределенные по полям, объявленым в заголовке таблицы
///
/// ```ignore
///  Name   |  Age | City
///  id1    | id2  | id3
///  ----   | ---- | ----
///   Thom  |  12  | LA
///   Jery  |   7  | LA
/// ```
pub struct CsvTable {
    pub header: Vec<CsvField>,
    rows: Vec<FxIndexMap<String, String>>,
    dbg: Dbg,
}
//
#[allow(unused)]
impl CsvTable {
    /// ### Returns empty `CsvTable` new instance
    pub fn new(prnt: impl Into<String>) -> Self {
        let dbg = Dbg::new(prnt, crate::me::<Self>());
        Self {
            header: vec![],
            rows: vec![],
            dbg,
        }
    }
    /// ### Loads table from the csv file
    #[named]
    pub fn load(parent: impl Into<String>, path: impl AsRef<Path>) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, crate::me::<Self>());
        let rdr = OpenOptions::new().read(true).open(path.as_ref()).map_err(|err| err_pass!(dbg, err, "Can't open file '{}'", path.as_ref().display()))?;
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(false)
            .has_headers(false)
            .from_reader(rdr);
        // log::debug!("{dbg} | Reading csv data from '{:?}'...", path.as_ref());
        let csv: csv::DeserializeRecordsIter<'_, _, Record> = rdr.deserialize();
        let mut csv_rows = csv.enumerate();
        let header = Self::read_header(&dbg, &mut csv_rows).map_err(|err| err_pass!(dbg, err))?;
        let mut rows = vec![];
        while let Some((i, csv_row)) = csv_rows.next() {
            match csv_row {
                Ok(csv_row) => {
                    let mut row = FxIndexMap::with_capacity(csv_row.len());
                    for field in &header {
                        let Some(cell) = csv_row.get(field.ix).cloned() else {
                            return Err(err!(&dbg, "Row [{i}] has {} fields, expected {}", csv_row.len(), header.len()));
                        };
                        row.insert(field.key.clone(), cell);
                    }
                    rows.push(row);
                }
                Err(err) => return Err(err_pass!(dbg, err, "Can't read row [{i}]")),
            }
        }
        Ok(Self {
            header,
            rows,
            dbg,
        })
    }
    /// ### Читает заголовок таблицы
    /// будет искать:
    /// - Одну строку с идентификаторами полей (столбцов)
    /// - Или две строки с идентификаторами полей (столбцов) и коментариями над ними
    #[named]
    fn read_header(dbg: &Dbg, rows: &mut impl Iterator<Item = (usize, Result<Record, csv::Error>)>) -> Result<Header, Error> {
        while let Some((i, row)) = rows.next() {
            match row {
                Ok(row) => {
                    let is_header = Self::is_header(HEADER_KEYS, &row).map_err(|err| err_pass!(dbg, err))?;
                    // log::debug!("{dbg} | row [{i}]: {} : {:?}..", if is_header { "HEDER" } else { "NOT a HEDER" }, row.get(..8).unwrap_or(&row));
                    if is_header {
                        let header = row.into_iter().enumerate().map(|(ix, key)| CsvField { key, ix }).collect();
                        return Ok(header);
                    }
                },
                Err(err) => log::warn!("{dbg} | Can't read row [{i}]: {:?}", err),
            };
        }
        Err(err!(&dbg, "Can't find header {:?}... etc.", HEADER_KEYS.get(..8).unwrap_or(HEADER_KEYS)))
    }
    /// Определяет заголовок таблицы
    /// - `required_fields` - Поля, обязательные для заголовка.
    /// - `row` - Строка таблицы.
    /// - Возвращает Ok(true) если это заголовок и все необходимые поля есть.
    /// - Возвращает Err если это заголовок, но поля не все.
    /// - Иначе возвращает Ok(false) - это не заголовок.
    #[named]
    fn is_header<S: AsRef<str>>(required_fields: &[S], row: &[String]) -> Result<bool, Error> {
        let missing_fields: Vec<&str> = required_fields
            .iter()
            .filter(|f| !row.iter().any(|cell| f.as_ref() == cell.as_str()))
            .map(|f| f.as_ref())
            .collect();
        if missing_fields.is_empty() {
            return Ok(true)
        }
        if missing_fields.len() == required_fields.len() {
            return Ok(false)
        }
        Err(err!(Self, "Fields missed in the header: {:?}", missing_fields))
    }
    /// ### Returns row by field `key`
    pub fn row(&self, row: usize) -> Option<&FxIndexMap<String, String>> {
        self.rows.get(row)
    }
    /// ### Returns value by field `key` and row index
    #[named]
    pub fn cell<T: FromStr>(&self, row: usize, key: impl AsRef<str>) -> Result<Option<T>, Error>
    where
        <T as FromStr>::Err: std::fmt::Display {
        let row = self.rows.get(row).ok_or_else(|| err!(self.dbg, "row {row} is out of bounds 0..{}", self.rows.len()))?;
        let val = row.get(key.as_ref()).ok_or_else(|| err!(self.dbg, "key {} is not found in table fields", key.as_ref()))?;
        val.parse::<T>().map(|v| Some(v)).map_err(|err| err_pass!(self.dbg, err))
    }
    /// ### Returns number of rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    /// ### Returns number of fields (columns)
    pub fn fields(&self) -> usize {
        self.header.len()
    }
    /// Returns iterator over the table rows references
    pub fn iter(&self) -> impl Iterator<Item = super::CsvRowRef<'_>> {
        self.rows.iter().map(|cells| super::CsvRowRef { cells })
    }
}
//
impl IntoIterator for CsvTable {
    type Item = CsvRow;
    type IntoIter = std::iter::Map<std::vec::IntoIter<FxIndexMap<String, String>>, fn(FxIndexMap<String, String>) -> CsvRow>;
    /// Returns iterator over the table rows
    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter().map(|cells| CsvRow { cells })
    }
}

/// Обязательные поля таблицы
static HEADER_KEYS: &[&str] = &[
    "step",
    "x_nok","y_nok",
    "xg","yg",
    "x_hook","y_hook",
    "lrope_hook_min","lrope_straight","lrope_ark","lrope_winch",
    "l_rope","l_rope_winch_geom","l_rope_winch",
    "pos",
];
