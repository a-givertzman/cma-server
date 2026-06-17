use sal_core::error::Error;

///
/// ### Вспомогательный слой для операций с числами
/// - Обеспечивает строгую типизацию, перекрестное приведение типов при вычислениях
/// - Защищает от переполнений и распространения `NaN` в математическом ядре графа.
#[derive(Debug, Clone, Copy)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Real(f32),
    Double(f64),
}
//
impl TryFrom<sal_sync::services::entity::Point> for Value {
    type Error = Error;
    fn try_from(p: sal_sync::services::entity::Point) -> Result<Self, Self::Error> {
        TryFrom::<&sal_sync::services::entity::Point>::try_from(&p)
    }
}
//
impl TryFrom<&sal_sync::services::entity::Point> for Value {
    type Error = Error;
    fn try_from(p: &sal_sync::services::entity::Point) -> Result<Self, Self::Error> {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Ok(Self::Bool(p.value.0)),
            sal_sync::services::entity::Point::Int(p) => Ok(Self::Int(p.value)),
            sal_sync::services::entity::Point::Real(p) => Ok(Self::Real(p.value)),
            sal_sync::services::entity::Point::Double(p) => Ok(Self::Double(p.value)),
            _ => return Err(Self::Error::new("Value", "try_from").err(concat_string::concat_string!("Invalid type '", p.typ().to_string(), "'"))),
        }
    }
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
    pub fn pow(self, exp: Self) -> Result<Value, Error> {
        if (self.is_bool() || self.is_int()) && exp.is_negative() {
            return Err(Error::new("Value", "pow").err(format!("Exponent can't be negative for integer base: `{:?} ^ {:?}`", self, exp)));
        }
        if self.is_nan() || exp.is_nan() {
            return Err(Error::new("Value", "pow").err(format!("Invalid input: `{:?} ^ {:?}`", self, exp)));
        }
        let exp_as_u32 = |v: i64| -> Result<u32, Error> {
            u32::try_from(v).map_err(|_| Error::new("Value", "pow").err(format!("Exponent too large for integer pow: {}", v)))
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
    ///
    /// ### Boolean `Or`, operation `||`
    /// 
    /// Правила приведения типов
    /// * `Bool` - в исходном виде.
    /// * `Int`, `Real`, `Double` - значения `0` и `0.0` интерпретируются как `false`, ненулевые значения (включая отрицательные) интерпретируются как `true`.
    ///
    /// # Ошибки
    /// Возвращает `Err`, если одно из значений является `NaN`, предотвращая распространение недостоверных данных в графе.
    pub fn or(&self, other: &Value) -> Result<bool, Error> {
        if self.is_nan() || other.is_nan() {
            return Err(Error::new("Value", "or").err(format!("Invalid input: `{:?} || {:?}`", self, other)));
        }
        match (self, other) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(*v1 || *v2),
            (Value::Bool(v1), Value::Int(v2)) => Ok(*v1 || (*v2 != 0)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(*v1 || (*v2 != 0.0)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(*v1 || (*v2 != 0.0)),
            (Value::Int(v1), Value::Bool(v2)) => Ok((*v1 != 0) || *v2),
            (Value::Int(v1), Value::Int(v2)) => Ok((*v1 != 0) || (*v2 != 0)),
            (Value::Int(v1), Value::Real(v2)) => Ok((*v1 != 0) || (*v2 != 0.0)),
            (Value::Int(v1), Value::Double(v2)) => Ok((*v1 != 0) || (*v2 != 0.0)),
            (Value::Real(v1), Value::Bool(v2)) => Ok((*v1 != 0.0) || *v2),
            (Value::Real(v1), Value::Int(v2)) => Ok((*v1 != 0.0) || (*v2 != 0)),
            (Value::Real(v1), Value::Real(v2)) => Ok((*v1 != 0.0) || (*v2 != 0.0)),
            (Value::Real(v1), Value::Double(v2)) => Ok((*v1 != 0.0) || (*v2 != 0.0)),
            (Value::Double(v1), Value::Bool(v2)) => Ok((*v1 != 0.0) || *v2),
            (Value::Double(v1), Value::Int(v2)) => Ok((*v1 != 0.0) || (*v2 != 0)),
            (Value::Double(v1), Value::Real(v2)) => Ok((*v1 != 0.0) || (*v2 != 0.0)),
            (Value::Double(v1), Value::Double(v2)) => Ok((*v1 != 0.0) || (*v2 != 0.0)),
        }
    }
    ///
    /// ### Boolean `And`, operation `&&`
    /// 
    /// Правила приведения типов
    /// * `Bool` - в исходном виде.
    /// * `Int`, `Real`, `Double` - значения `0` и `0.0` интерпретируются как `false`, ненулевые значения (включая отрицательные) интерпретируются как `true`.
    ///
    /// # Ошибки
    /// Возвращает `Err`, если одно из значений является `NaN`, предотвращая распространение недостоверных данных в графе.
    pub fn and(&self, other: &Value) -> Result<bool, Error> {
        if self.is_nan() || other.is_nan() {
            return Err(Error::new("Value", "and").err(format!("Invalid input: `{:?} && {:?}`", self, other)));
        }
        match (self, other) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(*v1 && *v2),
            (Value::Bool(v1), Value::Int(v2)) => Ok(*v1 && (*v2 != 0)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(*v1 && (*v2 != 0.0)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(*v1 && (*v2 != 0.0)),
            (Value::Int(v1), Value::Bool(v2)) => Ok((*v1 != 0) && *v2),
            (Value::Int(v1), Value::Int(v2)) => Ok((*v1 != 0) && (*v2 != 0)),
            (Value::Int(v1), Value::Real(v2)) => Ok((*v1 != 0) && (*v2 != 0.0)),
            (Value::Int(v1), Value::Double(v2)) => Ok((*v1 != 0) && (*v2 != 0.0)),
            (Value::Real(v1), Value::Bool(v2)) => Ok((*v1 != 0.0) && *v2),
            (Value::Real(v1), Value::Int(v2)) => Ok((*v1 != 0.0) && (*v2 != 0)),
            (Value::Real(v1), Value::Real(v2)) => Ok((*v1 != 0.0) && (*v2 != 0.0)),
            (Value::Real(v1), Value::Double(v2)) => Ok((*v1 != 0.0) && (*v2 != 0.0)),
            (Value::Double(v1), Value::Bool(v2)) => Ok((*v1 != 0.0) && *v2),
            (Value::Double(v1), Value::Int(v2)) => Ok((*v1 != 0.0) && (*v2 != 0)),
            (Value::Double(v1), Value::Real(v2)) => Ok((*v1 != 0.0) && (*v2 != 0.0)),
            (Value::Double(v1), Value::Double(v2)) => Ok((*v1 != 0.0) && (*v2 != 0.0)),
        }
    }
}
//
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(v1), Value::Bool(v2)) => v1 == v2,
            (Value::Bool(_), _) => false,
            (_, Value::Bool(_)) => false,
            (Value::Int(v1), Value::Int(v2)) => v1 == v2,
            (Value::Int(v1), Value::Real(v2)) => eq_i64_f64(*v1, *v2 as f64),
            (Value::Int(v1), Value::Double(v2)) => eq_i64_f64(*v1, *v2),
            (Value::Real(v1), Value::Int(v2)) => eq_i64_f64(*v2, *v1 as f64),
            (Value::Real(v1), Value::Real(v2)) => v1 == v2,
            (Value::Real(v1), Value::Double(v2)) => (*v1 as f64) == *v2,
            (Value::Double(v1), Value::Int(v2)) => eq_i64_f64(*v2, *v1),
            (Value::Double(v1), Value::Real(v2)) => *v1 == (*v2 as f64),
            (Value::Double(v1), Value::Double(v2)) => v1 == v2,
        }
    }
}
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Bool(v1), Value::Bool(v2)) => v1.partial_cmp(v2),
            (Value::Bool(_), _) => None,
            (_, Value::Bool(_)) => None,
            (Value::Int(v1), Value::Int(v2)) => v1.partial_cmp(v2),
            (Value::Int(v1), Value::Real(v2)) => cmp_i64_f64(*v1, *v2 as f64),
            (Value::Int(v1), Value::Double(v2)) => cmp_i64_f64(*v1, *v2),
            (Value::Real(v1), Value::Int(v2)) => cmp_i64_f64(*v2, *v1 as f64).map(|o| o.reverse()),
            (Value::Real(v1), Value::Real(v2)) => v1.partial_cmp(v2),
            (Value::Real(v1), Value::Double(v2)) => (*v1 as f64).partial_cmp(v2),
            (Value::Double(v1), Value::Int(v2)) => cmp_i64_f64(*v2, *v1).map(|o| o.reverse()),
            (Value::Double(v1), Value::Real(v2)) => v1.partial_cmp(&(*v2 as f64)),
            (Value::Double(v1), Value::Double(v2)) => v1.partial_cmp(v2),
        }
    }
}
/// Вспомогательное строгое сравнение целого i64 и вещественного f64 без потери точности мантиссы.
fn eq_i64_f64(i: i64, f: f64) -> bool {
    if f >= -9223372036854775808.0 && f < 9223372036854775808.0 {
        f as i64 == i && (i as f64 == f)
    } else {
        false
    }
}
/// Вспомогательное упорядочивание целого i64 и вещественного f64 с защитой от усечения разрядов.
fn cmp_i64_f64(i: i64, f: f64) -> Option<std::cmp::Ordering> {
    if f.is_nan() {
        return None;
    }
    if f >= -9223372036854775808.0 && f < 9223372036854775808.0 {
        let f_int = f as i64;
        match i.cmp(&f_int) {
            std::cmp::Ordering::Equal => (i as f64).partial_cmp(&f),
            ord => Some(ord),
        }
    } else {
        (i as f64).partial_cmp(&f)
    }
}
//
// impl std::ops::BitAnd for Value {
//     type Output = Result<Value, Error>;
//     fn bitand(self, rhs: Self) -> Self::Output {
//         if self.is_nan() || rhs.is_nan() {
//             return Err(Error::new("Value", "add").err(format!("Value.add | Invalid input: `{:?} + {:?}`", self, rhs)));
//         }
//         match (self, rhs) {
//             (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Bool(v1 | v2)),
//             (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64) | v2)),
//             (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 | v2)),
//             (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 | v2)),
//             (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_add(v2 as i64).ok_or_else(|| Error::new("Value", "add").err(format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2)))?)),
//             (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_add(v2).ok_or_else(|| Error::new("Value", "add").err(format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2)))?)),
//             (Value::Int(v1), Value::Real(v2)) => Ok(Value::Real(v1 as f32 + v2)),
//             (Value::Int(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 + v2)),
//             (Value::Real(v1), Value::Bool(v2)) => Ok(Value::Real(v1 + (v2 as u8) as f32)),
//             (Value::Real(v1), Value::Int(v2)) => Ok(Value::Real(v1 + v2 as f32)),
//             (Value::Real(v1), Value::Real(v2)) => Ok(Value::Real(v1 + v2)),
//             (Value::Real(v1), Value::Double(v2)) => Ok(Value::Double(v1 as f64 + v2)),
//             (Value::Double(v1), Value::Bool(v2)) => Ok(Value::Double(v1 + (v2 as u8) as f64)),
//             (Value::Double(v1), Value::Int(v2)) => Ok(Value::Double(v1 + v2 as f64)),
//             (Value::Double(v1), Value::Real(v2)) => Ok(Value::Double(v1 + v2 as f64)),
//             (Value::Double(v1), Value::Double(v2)) => Ok(Value::Double(v1 + v2)),
//         }
//     }
// }
//
impl std::ops::Add for Value {
    type Output = Result<Value, Error>;
    fn add(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Err(Error::new("Value", "add").err(format!("Value.add | Invalid input: `{:?} + {:?}`", self, rhs)));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 + v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_add(v2).ok_or_else(|| Error::new("Value", "add").err(format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2)))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 + v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 + v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_add(v2 as i64).ok_or_else(|| Error::new("Value", "add").err(format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2)))?)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_add(v2).ok_or_else(|| Error::new("Value", "add").err(format!("Value.add | Overflow: `{:?} + {:?}`", v1, v2)))?)),
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
    type Output = Result<Value, Error>;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Err(Error::new("Value", "sub").err(format!("Value.sub | Invalid input: `{:?} - {:?}`", self, rhs)));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 - v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_sub(v2).ok_or_else(|| Error::new("Value", "sub").err(format!("Value.sub | Overflow: `{:?} - {:?}`", v1, v2)))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 - v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 - v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_sub(v2 as i64).ok_or_else(|| Error::new("Value", "sub").err(format!("Value.sub | Overflow: `{:?} - {:?}`", v1, v2)))?)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_sub(v2).ok_or_else(|| Error::new("Value", "sub").err(format!("Value.sub | Overflow: `{:?} - {:?}`", v1, v2)))?)),
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
    type Output = Result<Value, Error>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            return Err(Error::new("Value", "div").err(format!("Value.div | Division by zero: `{:?} / {:?}`", self, rhs)));
        }
        if self.is_nan() || rhs.is_nan() {
            return Err(Error::new("Value", "div").err(format!("Value.div | Invalid input: `{:?} / {:?}`", self, rhs)));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 / v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int(v1 as i64 / v2)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 / v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 / v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1 / v2 as i64)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_div(v2).ok_or_else(|| Error::new("Value", "div").err(format!("Value.div | Overflow: `{:?} / {:?}`", v1, v2)))?)),
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
    type Output = Result<Value, Error>;
    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            return Err(Error::new("Value", "mul").err(format!("Value.mul | Invalid input: `{:?} * {:?}`", self, rhs)));
        }
        match (self, rhs) {
            (Value::Bool(v1), Value::Bool(v2)) => Ok(Value::Int(v1 as i64 * v2 as i64)),
            (Value::Bool(v1), Value::Int(v2)) => Ok(Value::Int((v1 as i64).checked_mul(v2).ok_or_else(|| Error::new("Value", "mul").err(format!("Value.mul | Overflow: `{:?} * {:?}`", v1, v2)))?)),
            (Value::Bool(v1), Value::Real(v2)) => Ok(Value::Real((v1 as u8) as f32 * v2)),
            (Value::Bool(v1), Value::Double(v2)) => Ok(Value::Double((v1 as u8) as f64 * v2)),
            (Value::Int(v1), Value::Bool(v2)) => Ok(Value::Int(v1.checked_mul(v2 as i64).ok_or_else(|| Error::new("Value", "mul").err(format!("Value.mul | Overflow: `{:?} * {:?}`", v1, v2)))?)),
            (Value::Int(v1), Value::Int(v2)) => Ok(Value::Int(v1.checked_mul(v2).ok_or_else(|| Error::new("Value", "mul").err(format!("Value.mul | Overflow: `{:?} * {:?}`", v1, v2)))?)),
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
    fn test_or_pure_boolean() {
        let t = Value::Bool(true);
        let f = Value::Bool(false);
        assert_eq!(t.or(&f).unwrap(), true);
        assert_eq!(f.or(&f).unwrap(), false);
    }
    #[test]
    fn test_or_negative_numeric_truthiness() {
        let f = Value::Bool(false);
        let i_neg = Value::Int(-1);
        let r_neg = Value::Real(-10.5);
        let d_zero = Value::Double(0.0);
        assert_eq!(f.or(&i_neg).unwrap(), true);
        assert_eq!(f.or(&r_neg).unwrap(), true);
        assert_eq!(f.or(&d_zero).unwrap(), false);
    }
    #[test]
    fn test_or_strict_nan_poisoning() {
        let t = Value::Bool(true);
        let nan_val = Value::Double(f64::NAN);
        assert!(t.or(&nan_val).is_err());
    }
    #[test]
    fn test_cross_type_equality() {
        assert_eq!(Value::Int(42), Value::Double(42.0));
        assert_eq!(Value::Real(10.5), Value::Double(10.5));
        assert_ne!(Value::Int(42), Value::Bool(true));
    }
    #[test]
    fn test_cross_type_ordering() {
        assert!(Value::Double(10.5) > Value::Int(5));
        assert!(Value::Int(-10) < Value::Real(0.0));
        assert_eq!(Value::Bool(true).partial_cmp(&Value::Int(10)), None);
    }
    #[test]
    fn test_nan_comparison_safety() {
        let nan_val = Value::Double(f64::NAN);
        let num_val = Value::Int(10);
        assert_ne!(nan_val, num_val);
        assert_eq!(nan_val.partial_cmp(&num_val), None);
    }
    #[test]
    fn test_cross_type_strict_equality() {
        assert!(Value::Int(1) != Value::Bool(true));
        assert!(Value::Int(0) != Value::Bool(false));
        assert!(Value::Real(1.0) != Value::Bool(true));
        assert_eq!(Value::Int(42), Value::Real(42.0));
        assert_eq!(Value::Int(100), Value::Double(100.0));
        assert_ne!(Value::Int(100), Value::Real(100.05));
    }
    #[test]
    fn test_large_values_safety() {
        assert_eq!(Value::Int(150_000), Value::Real(150_000.0));
        assert_ne!(Value::Int(16_777_217), Value::Real(16_777_216.0));
        assert_ne!(Value::Int(16_777_217), Value::Double(16_777_216.0));
        assert_ne!(Value::Real(16_777_216.0), Value::Int(16_777_217));
        assert_ne!(Value::Double(16_777_216.0), Value::Int(16_777_217));
    }
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
