#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnKind {
    Input,
    Var,
    Fn,
}