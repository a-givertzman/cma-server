use std::str::FromStr;
use function_name::named;
use sal_core::error::Error;
use crate::{domain::FxIndexMap, err, err_pass};

/// Строка таблицы `CsvTable`
pub struct CsvRow {
    pub(super) cells: FxIndexMap<String, String>
}
impl CsvRow {
    /// ### Returns value by field `key`
    #[named]
    pub fn cell<T: FromStr>(&self, key: impl AsRef<str>) -> Result<T, Error>
    where
        <T as FromStr>::Err: std::fmt::Display
    {
        let val = self.cells.get(key.as_ref()).ok_or_else(|| err!(Self, "key {} is not found in table fields", key.as_ref()))?;
        val.parse::<T>().map_err(|err| err_pass!(Self, err))
    }
    /// ### Returns f64 value by field `key`
    #[named]
    pub fn get_f64(&self, key: impl AsRef<str>) -> Result<f64, Error> {
        let val = self.cells.get(key.as_ref()).ok_or_else(|| err!(Self, "key {} is not found in table fields", key.as_ref()))?;
        val.parse::<f64>().map_err(|err| err_pass!(Self, err))
    }
    /// ### Returns usize value by field `key`
    #[named]
    pub fn get_usize(&self, key: impl AsRef<str>) -> Result<usize, Error> {
        let val = self.cells.get(key.as_ref()).ok_or_else(|| err!(Self, "key {} is not found in table fields", key.as_ref()))?;
        val.parse::<usize>().map_err(|err| err_pass!(Self, err))
    }
}
impl std::fmt::Debug for CsvRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (key, val)) in self.cells.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}[{}]", key, val)?;
        }
        Ok(())
    }
}
/// Ссылка на строку таблицы `CsvTable`
pub struct CsvRowRef<'a> {
    pub cells: &'a FxIndexMap<String, String>,
}
impl CsvRowRef<'_> {
    /// ### Returns value by field `key` and row index
    #[named]
    pub fn cell<T: FromStr>(&self, key: impl AsRef<str>) -> Result<T, Error>
    where
        <T as FromStr>::Err: std::fmt::Display
    {
        let val = self.cells.get(key.as_ref()).ok_or_else(|| err!(Self, "key {} is not found in table fields", key.as_ref()))?;
        val.parse::<T>().map_err(|err| err_pass!(Self, err))
    }
}
impl std::fmt::Debug for CsvRowRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (key, val)) in self.cells.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}[{}]", key, val)?;
        }
        Ok(())
    }
}
