#![allow(non_snake_case)]
use testing::entities::test_value::Value;

use crate::core_::point::point::{Point, ToPoint};

impl ToPoint for Value {
    fn to_point(&self, txId: usize, name: &str) -> Point {
        match self {
            Value::Bool(value) => value.to_point(txId, name),
            Value::Int(value) => value.to_point(txId, name),
            Value::Real(value) => value.to_point(txId, name),
            Value::Double(value) => value.to_point(txId, name),
            Value::String(value) => value.to_point(txId, name),
        }
    }
}