

pub enum Expr {
    Single { ch: char },
    Range {from: char, to: char},
    OrTable { variants: Vec<Expr> },
    Or { left: Box<Expr>, right: Box<Expr> },
    Seq { exprs: Vec<Expr> },
}

// TODO make enum
pub struct ParseError;

pub fn parse(text: &str) -> Result<Expr, ParseError> {
    unimplemented!()
}