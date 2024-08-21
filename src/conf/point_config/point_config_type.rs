use serde::{Serialize, Deserialize};

///
/// 
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointConfigType {
    #[serde(rename = "Bool")]
    #[serde(alias = "bool", alias = "Bool")]
    Bool,
    #[serde(rename = "Int")]
    #[serde(alias = "int", alias = "Int")]
    Int,
    #[serde(rename = "Real")]
    #[serde(alias = "real", alias = "Real")]
    Real,
    #[serde(rename = "Double")]
    #[serde(alias = "double", alias = "Double")]
    Double,
    #[serde(rename = "String")]
    #[serde(alias = "string", alias = "String")]
    String,
    #[serde(rename = "Json")]
    #[serde(alias = "json", alias = "Json")]
    Json,
}
//
//
impl ToString for PointConfigType {
    fn to_string(&self) -> String {
        match self {
            PointConfigType::Bool => "Bool".to_owned(),
            PointConfigType::Int => "Int".to_owned(),
            PointConfigType::Real => "Real".to_owned(),
            PointConfigType::Double => "Double".to_owned(),
            PointConfigType::String => "String".to_owned(),
            PointConfigType::Json => "Json".to_owned(),
        }
    }
}
