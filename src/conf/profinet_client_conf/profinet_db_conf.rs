use sal_core::error::Error;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf, PointType}, task::functions::{FnConfKeywd, FnConfKindName}};
use std::str::FromStr;
///
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct ProfinetDbConf {
    pub(crate) name: Name,
    pub(crate) description: String,
    pub(crate) number: u64,
    pub(crate) offset: u64,
    pub(crate) size: u64,
    pub(crate) points: Vec<PointConf>,
}
//
// 
impl ProfinetDbConf {
    ///
    /// Returns [ProfinetDbConf] new instance
    pub fn new(parent: impl Into<String>, name: &str, conf: ConfTree) -> Self {
        log::trace!("ProfinetDbConf.new | conf: {:?}", conf);
        let dbg = format!("ProfinetDbConf({})", name);
        let name = Name::new(parent, name);
        log::debug!("{}.new | name: {:?}", dbg, name);
        let description = conf.get("description").unwrap_or(String::new());
        log::debug!("{}.new | description: {:?}", dbg, description);
        let number = conf.get("number").unwrap();
        log::debug!("{}.new | number: {:?}", dbg, number);
        let offset = conf.get("offset").unwrap();
        log::debug!("{}.new | offset: {:?}", dbg, offset);
        // let size = conf.get("size").unwrap();
        // log::debug!("{}.new | size: {:?}", dbg, size);
        let mut points: Vec<PointConf> = vec![];
        for key in conf.keys(&["description", "number", "offset", "size"]) {
            let keyword = FnConfKeywd::from_str(&key).unwrap();
            if keyword.kind() == FnConfKindName::Point {
                let point_name = format!("{}/{}", name, keyword.data());
                let point_conf = conf.get(key).unwrap();
                log::trace!("{}.new | Point '{}'", dbg, point_name);
                log::trace!("{}.new | Point '{}'   |   conf: {:?}", dbg, point_name, point_conf);
                let point = PointConf::new(&name, &point_conf);
                points.push(point.clone());
            } else {
                log::debug!("{}.new | Device expected, but found {:?}", dbg, keyword);
            }
        }
        let size = Self::validate_addresses(&points)
            .map_err(|err| Error::new(&dbg, "new").pass(err.to_string())).unwrap() as u64;
        Self {
            name,
            description,
            number,
            offset,
            size,
            points,
        }
    }    
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
        self.points.iter().fold(vec![], |mut points, conf| {
            points.push(conf.clone());
            points
        })
    }
    ///
    /// Returns the PROFINET size of the type
    fn profinet_size(typ: &PointType) -> Option<u32> {
        match typ {
            PointType::Bool => Some(2),
            PointType::Int => Some(2),
            PointType::Real => Some(4),
            _ => None,
        }
    }
    ///
    /// Validates the adressing consistance
    fn validate_addresses(points: &Vec<PointConf>) -> Result<u32, AddressError> {
        if points.is_empty() {
            return Ok(0);
        }
        let mut points = points.clone();
        // 1. Сортируем: сначала по офсету, затем по биту
        // Используем стабильную сортировку, чтобы сохранить порядок именования
        points.sort_by(|a, b| {
            let addr_a = a.address.as_ref();
            let addr_b = b.address.as_ref();
            let off_a = addr_a.and_then(|at| at.offset).unwrap_or(0);
            let off_b = addr_b.and_then(|at| at.offset).unwrap_or(0);
            if off_a != off_b {
                off_a.cmp(&off_b)
            } else {
                let bit_a = addr_a.and_then(|at| at.bit).unwrap_or(0);
                let bit_b = addr_b.and_then(|at| at.bit).unwrap_or(0);
                bit_a.cmp(&bit_b)
            }
        });
        let mut total_size: u32 = 0;
        // Храним данные о предыдущем обработанном сигнале для сравнения
        let mut last_offset: u32 = 0;
        let mut last_end: u32 = 0;
        let mut last_bit: Option<u8> = None;
        let mut last: Option<PointConf> = None;
        for p in points.iter() {
            let addr = p.address.as_ref().ok_or_else(|| {
                log::error!("Point '{}' has no address", p.name);
                AddressError::MissingAddress(p.name.clone())
            })?;
            let offset = addr.offset.ok_or_else(|| AddressError::MissingAddress(p.name.clone()))?;
            let bit = addr.bit;
            let size = Self::profinet_size(&p.type_).unwrap_or(0);
            // --- ПРОВЕРКА ВЫРАВНИВАНИЯ (Siemens Best Practice) ---
            if size > 1 && offset % 2 != 0 {
                log::warn!("Performance warning: Point '{}' ({:?}) offset {} is not even (unaligned)", p.name, p.type_, offset);
            }
            // --- ПРОВЕРКА ПЕРЕКРЫТИЯ (Overlap) ---
            if offset < last_end {
                // Исключение: если оба сигнала - Bool и имеют один и тот же offset
                if let (PointType::Bool, Some(b)) = (p.type_.clone(), bit) {
                    if offset == last_offset {
                        // Проверяем последовательность бит
                        if let Some(lb) = last_bit {
                            if b <= lb {
                                log::error!("Point '{}' bit {} repeats or goes backwards (last bit was {})", p.name, b, lb);
                                return Err(AddressError::InvalidBit(p.name.clone(), b));
                            }
                        }
                        if b > 15 { // Т.к. profinet_size для Bool = 2 байта
                            log::error!("Point '{}' bit {} out of range (0-15)", p.name, b);
                            return Err(AddressError::InvalidBit(p.name.clone(), b));
                        }
                        // Это корректная упаковка бит, не считаем ошибкой перекрытия
                    } else {
                        // Офсет сменился, но залез на хвост предыдущего
                        if let Some(last) = last {
                            log::error!("Point '{}' (offset {}) overlaps with '{}' (ends at {})", p.name, offset, last.name, last_end);
                            return Err(AddressError::Overlap(p.clone(), last.clone()));
                        }
                    }
                } else {
                    // Это не Bool или офсеты пересекаются некорректно
                    if let Some(last) = last {
                        log::error!("Point '{}' (offset {}) overlaps with '{}' (ends at {})", p.name, offset, last.name, last_end);
                        return Err(AddressError::Overlap(p.clone(), last.clone()));
                    }
                }
            }
            // Обновляем состояние
            last_offset = offset;
            last_end = offset + size;
            last_bit = bit;
            last = Some(p.clone());
            if last_end > total_size {
                total_size = last_end;
            }
            log::trace!("Point validated: {} at {}.{}", p.name, offset, bit.unwrap_or(0));
        }
        Ok(total_size)
    }
}
///
/// 
enum AddressError {
    Overlap(PointConf, PointConf),
    InvalidBit(String, u8),
    MissingAddress(String),
}
impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressError::Overlap(p1, p2) => write!(f, "Overlap between: \n\t'{}':{:?} and \n\t'{}':{:?}", p1.name, p1.address, p2.name, p2.address),
            AddressError::InvalidBit(p, b) => write!(f, "Invalid bit index {} for point '{}'", b, p),
            AddressError::MissingAddress(p) => write!(f, "Address missing for point '{}'", p),
        }
    }
}
