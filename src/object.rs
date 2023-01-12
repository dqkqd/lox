pub(crate) type Number = f64;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Object {
    Null,
    Number(Number),
    String(String),
    Bool(bool),
}
