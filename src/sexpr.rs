#[derive(Debug, PartialEq)]
pub enum SExpr {
    Nil,
    List(Vec<SExpr>),
    Id(String),
    Str(String),
}
