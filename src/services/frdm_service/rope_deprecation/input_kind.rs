///
/// Variants of the service input
/// - Const: ConfDistance
/// - Point: point real 'App/MultiQueue/Load.MainBoomAngle'
#[derive(Debug, Clone, PartialEq)]
pub enum InputKind<T> {
    Const(T),
    Point(String),
}