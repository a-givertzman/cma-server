///
/// Helper `Value` to simplify the math operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Real(f32),
    Double(f64),
}
//
impl Value {
    pub fn is_nan(&self) -> bool {
        match self {
            Value::Bool(_) => false,
            Value::Int(_) => false,
            Value::Real(v) => v.is_nan(),
            Value::Double(v) => v.is_nan(),
        }
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Bool(v) => !v,
            Value::Int(v) => *v == 0,
            Value::Real(v) => v.abs() < f32::EPSILON,
            Value::Double(v) => v.abs() < f64::EPSILON,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            Value::Int(_) => true,
            _ => false,
        }
    }
    pub fn is_real(&self) -> bool {
        match self {
            Value::Real(_) => true,
            _ => false,
        }
    }
    pub fn is_double(&self) -> bool {
        match self {
            Value::Double(_) => true,
            _ => false,
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            Value::Bool(_) => false,
            Value::Int(v) => v.is_negative(),
            Value::Real(v) => v.is_sign_negative(),
            Value::Double(v) => v.is_sign_negative(),
        }
    }
    pub fn pow(self, exp: Self) -> Result<Value, String> {
        if (self.is_bool() || self.is_int()) && exp.is_negative() {
            return Err(format!("Value.pow | Exponent can't be negative for integer base: `{:?} ^ {:?}`", self, exp));
        }
        if self.is_nan() || exp.is_nan() {
            return Err(format!("Value.pow | Invalid input: `{:?} ^ {:?}`", self, exp));
        }
        let exp_as_u32 = |v: i64| -> Result<u32, String> {
            u32::try_from(v).map_err(|_| format!("Value.pow | Exponent too large for integer pow: {}", v))
        };
        match (self, exp) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int((v1 as i64).pow(v2 as u32))),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_pow(exp_as_u32(v2)?).ok_or_else(|| format!("Value.pow | Overflow: `{:?} ^ {:?}`", v1, v2))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real(((v1 as u8) as f32).powf(v2))),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double(((v1 as u8) as f64).powf(v2))),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.pow(v2 as u32))),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_pow(exp_as_u32(v2)?).ok_or_else(|| format!("Value.pow | Overflow: `{:?} ^ {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real((v1 as f32).powf(v2))),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double((v1 as f64).powf(v2))),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1.powf((v2 as u8) as f32))),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1.powf(v2 as f32))),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1.powf(v2))),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double((v1 as f64).powf(v2))),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1.powf((v2 as u8) as f64))),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1.powf(v2 as f64))),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1.powf(v2 as f64))),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1.powf(v2))),
        }
    }
}
//
impl std::ops::Add for Value {
    type Output = Result<Value, String>;
    fn add(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Err(format!("Value.add | Invalid input: `{:?} + {:?}`", self, rhs));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 + v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_add(v2).ok_or_else(|| format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 + v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 + v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_add(v2 as i64).ok_or_else(|| format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_add(v2).ok_or_else(|| format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 + v2)),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 + v2)),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 + (v2 as u8) as f32)),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 + v2 as f32)),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 + v2)),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 + v2)),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 + (v2 as u8) as f64)),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 + v2 as f64)),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 + v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 + v2)),
        }
    }
}
//
impl std::ops::Sub for Value {
    type Output = Result<Value, String>;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Err(format!("Value.sub | Invalid input: `{:?} - {:?}`", self, rhs));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 - v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_sub(v2).ok_or_else(|| format!("Value.sub | Overflow: `{:?} - {:?}`", v1, v2))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 - v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 - v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_sub(v2 as i64).ok_or_else(|| format!("Value.sub | Overflow: `{:?} - {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_sub(v2).ok_or_else(|| format!("Value.sub | Overflow: `{:?} - {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 - v2)),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 - v2)),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 - (v2 as u8) as f32)),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 - v2 as f32)),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 - v2)),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 - v2)),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 - (v2 as u8) as f64)),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 - v2 as f64)),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 - v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 - v2)),
        }
    }
}
//
impl std::ops::Div for Value {
    type Output = Result<Value, String>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            return Err(format!("Value.div | Division by zero: `{:?} / {:?}`", self, rhs));
        }
        if self.is_nan() || rhs.is_nan() {
            return Err(format!("Value.div | Invalid input: `{:?} / {:?}`", self, rhs));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 / v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int(v1 as i64 / v2)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 / v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 / v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1 / v2 as i64)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_div(v2).ok_or_else(|| format!("Value.div | Overflow: `{:?} / {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 / v2)),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 / v2)),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 / (v2 as u8) as f32)),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 / v2 as f32)),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 / v2)),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 / v2)),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 / (v2 as u8) as f64)),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 / v2 as f64)),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 / v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 / v2)),
        }
    }
}
//
impl std::ops::Mul for Value {
    type Output = Result<Value, String>;
    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Err(format!("Value.mul | Invalid input: `{:?} * {:?}`", self, rhs));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 * v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_mul(v2).ok_or_else(|| format!("Value.mul | Overflow: `{:?} * {:?}`", v1, v2))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 * v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 * v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_mul(v2 as i64).ok_or_else(|| format!("Value.mul | Overflow: `{:?} * {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_mul(v2).ok_or_else(|| format!("Value.mul | Overflow: `{:?} * {:?}`", v1, v2))?)),
            (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 * v2)),
            (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 * v2)),
            (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 * (v2 as u8) as f32)),
            (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 * v2 as f32)),
            (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 * v2)),
            (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 * v2)),
            (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 * (v2 as u8) as f64)),
            (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 * v2 as f64)),
            (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 * v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 * v2)),
        }
    }
}
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_zero_with_negative_floats() {
        let val1 = Value::Double(-5.5);
        let val2 = Value::Double(0.0);
        let val3 = Value::Double(-0.0);
        assert!(!val1.is_zero(), "Negative numbers are not zero");
        assert!(val2.is_zero(), "Positive zero is zero");
        assert!(val3.is_zero(), "Negative zero is zero");
    }
    #[test]
    fn test_add_type_promotion() {
        let bool_val = Value::Bool(true); // 1
        let int_val = Value::Int(5);
        let real_val = Value::Real(2.5);
        let double_val = Value::Double(10.5);
        let res1 = (bool_val + int_val).unwrap();
        assert!(matches!(res1, Value::Int(6)));
        let res2 = (int_val + real_val).unwrap();
        if let Value::Real(v) = res2 {
            assert_eq!(v, 7.5);
        } else {
            panic!("Expected Value::Real");
        }
        let res3 = (real_val + double_val).unwrap();
        if let Value::Double(v) = res3 {
            assert_eq!(v, 13.0);
        } else {
            panic!("Expected Value::Double");
        }
    }
    #[test]
    fn test_div_by_zero_prevention() {
        let val = Value::Int(10);
        let zero_int = Value::Int(0);
        let zero_float = Value::Double(0.0);
        assert!((val / zero_int).is_err());
        assert!((val / zero_float).is_err());
    }
    #[test]
    fn test_div_by_negative_number() {
        let val = Value::Double(10.0);
        let neg_float = Value::Double(-2.0);
        let res = (val / neg_float).unwrap();
        if let Value::Double(v) = res {
            assert_eq!(v, -5.0);
        } else {
            panic!("Expected Value::Double");
        }
    }
    #[test]
    fn test_overflow_protection() {
        let max_int = Value::Int(i64::MAX);
        let one = Value::Int(1);
        let minus_one = Value::Int(-1);
        assert!((max_int + one).is_err(), "Addition overflow should be caught");
        assert!((Value::Int(i64::MIN) / minus_one).is_err(), "Division overflow should be caught");
    }
    #[test]
    fn test_pow_logic() {
        let base = Value::Int(2);
        let exp_neg = Value::Int(-1);
        let exp_float = Value::Double(0.5);
        assert!(base.pow(exp_neg).is_err(), "Integer base with negative integer exp should error");
        let res2 = base.pow(exp_float).unwrap();
        if let Value::Double(v) = res2 {
            assert!((v - std::f64::consts::SQRT_2).abs() < f64::EPSILON);
        } else {
            panic!("Expected float promotion for float exponent");
        }
    }
}
