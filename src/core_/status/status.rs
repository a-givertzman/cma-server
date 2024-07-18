use std::{cmp::Ordering, str::FromStr};
use concat_string::concat_string;
///
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Status {
    Ok              = Self::OK as u32,
    Obsolete        = Self::OBSOLETE as u32,
    TimeInvalid     = Self::TIME_INVALID as u32,
    Invalid         = Self::INVALID as u32,
    Unknown(i64),
}
//
//
impl Status {
    const OK            : i64 = 0;
    const OBSOLETE      : i64 = 2;      // Prevously stored information always obsolete, connection lost
    const TIME_INVALID  : i64 = 3;
    const INVALID       : i64 = 10;     // Not sampled, conversion, calculation error
}
//
//
impl FromStr for Status {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "ok" => Ok(Status::Ok),
            "obsolete" => Ok(Status::Obsolete),
            "timeinvalid" => Ok(Status::TimeInvalid),
            "invalid" => Ok(Status::Invalid),
            _ => Err(concat_string!("Status.from_str | Unknown status '", input, "'")),
        }
    }
}
//
//
impl PartialOrd for Status {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
//
//
impl Ord for Status {
    fn cmp(&self, other: &Self) -> Ordering {
        Into::<u32>::into(*self).cmp(&Into::<u32>::into(*other))
    }
}
//
//
impl ToString for Status {
    fn to_string(&self) -> String {
        Into::<u32>::into(*self).to_string()
    }
}
//
//
impl From<i64> for Status {
    fn from(value: i64) -> Self {
        match value {
            Status::OK              => Status::Ok,
            Status::OBSOLETE        => Status::Obsolete,
            Status::TIME_INVALID    => Status::TimeInvalid,
            Status::INVALID         => Status::Invalid,
            _               => Status::Unknown(value),
        }
    }
}
//
//
impl From<u64> for Status {
    fn from(value: u64) -> Self {
        Self::from(value as i64)
    }
}
//
//
impl From<Status> for u64 {
    fn from(value: Status) -> Self {
        Into::<u32>::into(value) as u64
    }
}
//
//
impl From<Status> for u32 {
    fn from(value: Status) -> Self {
        match value {
            Status::Ok              => Status::OK as u32,
            Status::Obsolete        => Status::OBSOLETE as u32,
            Status::TimeInvalid     => Status::TIME_INVALID as u32,
            Status::Invalid         => Status::INVALID as u32,
            Status::Unknown(value) => value as u32,
        }
    }
}
//
//
impl From<Status> for i64 {
    fn from(value: Status) -> Self {
        match value {
            Status::Ok              => Status::OK,
            Status::Obsolete        => Status::OBSOLETE,
            Status::TimeInvalid     => Status::TIME_INVALID,
            Status::Invalid         => Status::INVALID,
            Status::Unknown(value) => value,
        }
    }
}
