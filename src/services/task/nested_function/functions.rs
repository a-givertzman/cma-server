//!
//! Here must be defined all functions to be awalible in the Task -> NestedFn
use std::str::FromStr;
use log::trace;
///
/// Entair list of public functions
/// supported by NestedFn builder
#[derive(Debug)]
pub enum Functions {
    /// embedded functions
    Input,
    Const,
    Var,
    /// debuging functions
    Debug,
    Plot,
    /// user defined functions
    Add,
    Count,
    Gt,
    Ge,
    Eq,
    Le,
    Lt,
    Ne,
    Timer,
    ToApiQueue,
    ToMultiQueue,
    SqlMetric,
    PointId,
    ToBool,
    ToInt,
    ToReal,
    ToDouble,
    ToString,
    Export,
    Filter,
    RisingEdge,
    FallingEdge,
    Retain,
    Acc,
    Mul,
    Div,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    Threshold,
    Smooth,
    Average,
    Pow,
    Max,
    PiecewiseLineApprox,
    IsChangedValue,
    KeepValid,
    /// Recorder functions
    RecOpCycleMetric,
}
//
//
impl Functions {
    /// embedded functions
    const INPUT                         : &'static str = "input";
    const CONST                         : &'static str = "const";
    const VAR                           : &'static str = "var";
    /// debuging functions
    const DEBUG                         : &'static str = "Debug";
    const PLOT                          : &'static str = "Plot";
    /// user defined functions
    const ADD                           : &'static str = "Add";
    const COUNT                         : &'static str = "Count";
    const GT                            : &'static str = "Gt";
    const GE                            : &'static str = "Ge";
    const EQ                            : &'static str = "Eq";
    const LE                            : &'static str = "Le";
    const LT                            : &'static str = "Lt";
    const NE                            : &'static str = "Ne";
    const TIMER                         : &'static str = "Timer";
    const TO_API_QUEUE                  : &'static str = "ToApiQueue";
    const TO_MULTI_QUEUE                : &'static str = "ToMultiQueue";
    const SQL_METRIC                    : &'static str = "SqlMetric";
    const POINT_ID                      : &'static str = "PointId";
    const TO_BOOL                       : &'static str = "ToBool";
    const TO_INT                        : &'static str = "ToInt";
    const TO_REAL                       : &'static str = "ToReal";
    const TO_DOUBLE                     : &'static str = "ToDouble";
    const TO_STRING                     : &'static str = "ToString";
    const EXPORT                        : &'static str = "Export";
    const FILTER                        : &'static str = "Filter";
    const RISING_EDGE                   : &'static str = "RisingEdge";
    const FALLING_EDGE                  : &'static str = "FallingEdge";
    const RETAIN                        : &'static str = "Retain";
    const ACC                           : &'static str = "Acc";
    const MUL                           : &'static str = "Mul";
    const DIV                           : &'static str = "Div";
    const SUB                           : &'static str = "Sub";
    const BIT_AND                       : &'static str = "BitAnd";
    const BIT_OR                        : &'static str = "BitOr";
    const BIT_XOR                       : &'static str = "BitXor";
    const BIT_NOT                       : &'static str = "BitNot";
    const THRESHOLD                     : &'static str = "Threshold";
    const SMOOTH                        : &'static str = "Smooth";
    const AVERAGE                       : &'static str = "Average";
    const POW                           : &'static str = "Pow";
    const MAX                           : &'static str = "Max";
    const PIECEWISE_LINE_APPROX         : &'static str = "PiecewiseLineApprox";
    const IS_CHANGED_VALUE              : &'static str = "IsChangedValue";
    const KEEP_VALID                    : &'static str = "KeepValid";
    /// Recorder functions
    const REC_OP_CYCLE_METRIC           : &'static str = "RecOpCycleMetric";
    ///
    /// Returns function name as string 
    pub fn name(&self) -> &str {
        match self {
            Self::Add                   => Self::ADD,
            Self::Const                 => Self::CONST,
            Self::Count                 => Self::COUNT,
            Self::Gt                    => Self::GT,
            Self::Ge                    => Self::GE,
            Self::Eq                    => Self::EQ,
            Self::Le                    => Self::LE,
            Self::Lt                    => Self::LT,
            Self::Ne                    => Self::NE,
            Self::Input                 => Self::INPUT,
            Self::Timer                 => Self::TIMER,
            Self::Var                   => Self::VAR,
            Self::ToApiQueue            => Self::TO_API_QUEUE,
            Self::ToMultiQueue          => Self::TO_MULTI_QUEUE,
            Self::SqlMetric             => Self::SQL_METRIC,
            Self::PointId               => Self::POINT_ID,
            Self::Debug                 => Self::DEBUG,
            Self::Plot                  => Self::PLOT,
            Self::ToBool                => Self::TO_BOOL,
            Self::ToInt                 => Self::TO_INT,
            Self::ToReal                => Self::TO_REAL,
            Self::ToDouble              => Self::TO_DOUBLE,
            Self::ToString              => Self::TO_STRING,
            Self::Export                => Self::EXPORT,
            Self::Filter                => Self::FILTER,
            Self::RisingEdge            => Self::RISING_EDGE,
            Self::FallingEdge           => Self::FALLING_EDGE,
            Self::Retain                => Self::RETAIN,
            Self::Acc                   => Self::ACC,
            Self::Mul                   => Self::MUL,
            Self::Div                   => Self::DIV,
            Self::Sub                   => Self::SUB,
            Self::BitAnd                => Self::BIT_AND,
            Self::BitOr                 => Self::BIT_OR,
            Self::BitXor                => Self::BIT_XOR,
            Self::BitNot                => Self::BIT_NOT,
            Self::Threshold             => Self::THRESHOLD,
            Self::Smooth                => Self::SMOOTH,
            Self::Average               => Self::AVERAGE,
            Self::Pow                   => Self::POW,
            Self::RecOpCycleMetric      => Self::REC_OP_CYCLE_METRIC,
            Self::Max                   => Self::MAX,
            Self::PiecewiseLineApprox   => Self::PIECEWISE_LINE_APPROX,
            Self::IsChangedValue        => Self::IS_CHANGED_VALUE,
            Self::KeepValid             => Self::KEEP_VALID,
        }
    }
    ///
    /// Returns enum Function corresponding to the function name
    fn match_name(input: &str) -> Result<Functions, String> {
        match input {
            Self::ADD                   => Ok( Self::Add ),
            Self::CONST                 => Ok( Self::Const ),
            Self::COUNT                 => Ok( Self::Count ),
            Self::GT                    => Ok( Self::Gt ),
            Self::GE                    => Ok( Self::Ge ),
            Self::EQ                    => Ok( Self::Eq ),
            Self::LE                    => Ok( Self::Le ),
            Self::LT                    => Ok( Self::Lt ),
            Self::NE                    => Ok( Self::Ne ),
            Self::INPUT                 => Ok( Self::Input ),
            Self::TIMER                 => Ok( Self::Timer ),
            Self::VAR                   => Ok( Self::Var ),
            Self::TO_API_QUEUE          => Ok( Self::ToApiQueue ),
            Self::TO_MULTI_QUEUE        => Ok( Self::ToMultiQueue ),
            Self::SQL_METRIC            => Ok( Self::SqlMetric ),
            Self::POINT_ID              => Ok( Self::PointId ),
            Self::DEBUG                 => Ok( Self::Debug ),
            Self::PLOT                  => Ok( Self::Plot ),
            Self::TO_BOOL               => Ok( Self::ToBool ),
            Self::TO_INT                => Ok( Self::ToInt ),
            Self::TO_REAL               => Ok( Self::ToReal ),
            Self::TO_DOUBLE             => Ok( Self::ToDouble ),
            Self::TO_STRING             => Ok( Self::ToString ),
            Self::EXPORT                => Ok( Self::Export ),
            Self::FILTER                => Ok( Self::Filter ),
            Self::RISING_EDGE           => Ok( Self::RisingEdge ),
            Self::FALLING_EDGE          => Ok( Self::FallingEdge ),
            Self::RETAIN                => Ok( Self::Retain ),
            Self::ACC                   => Ok( Self::Acc ),
            Self::MUL                   => Ok( Self::Mul ),
            Self::DIV                   => Ok( Self::Div ),
            Self::SUB                   => Ok( Self::Sub ),
            Self::BIT_AND               => Ok( Self::BitAnd ),
            Self::BIT_OR                => Ok( Self::BitOr ),
            Self::BIT_XOR               => Ok( Self::BitXor ),
            Self::BIT_NOT               => Ok( Self::BitNot ),
            Self::THRESHOLD             => Ok( Self::Threshold ),
            Self::SMOOTH                => Ok( Self::Smooth ),
            Self::AVERAGE               => Ok( Self::Average ),
            Self::POW                   => Ok( Self::Pow ),
            Self::REC_OP_CYCLE_METRIC   => Ok( Self::RecOpCycleMetric ),
            Self::MAX                   => Ok( Self::Max ),
            Self::PIECEWISE_LINE_APPROX => Ok( Self::PiecewiseLineApprox ),
            Self::IS_CHANGED_VALUE      => Ok( Self::IsChangedValue ),
            Self::KEEP_VALID            => Ok( Self::KeepValid ),
            _ => Err(format!("Functions.from_str | Unknown function name '{}'", &input)),
        }
    }
}
//
//
impl FromStr for Functions {
    type Err = String;
    fn from_str(input: &str) -> Result<Functions, String> {
        trace!("Functions.from_str | input: {}", input);
        Self::match_name(input)
    }
}
