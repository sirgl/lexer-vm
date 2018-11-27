

pub struct LexerDefinition {
    pub tokens: Vec<TokenDefinition>
}

impl LexerDefinition {
    pub fn new(tokens: Vec<TokenDefinition>) -> Self {
        LexerDefinition { tokens }
    }
}

#[derive(Clone)]
pub enum Expr {
    Single { ch: char },
    Range { from: char, to: char },
    Or { variants: Vec<Expr> },
    Seq { exprs: Vec<Expr> },
}

pub struct TokenDefinition {
    pub expr: Expr,
    pub index: u16,
    pub name: String
}

// TODO make enum
pub struct ParseError;

pub fn parse(text: &str) -> Result<Expr, ParseError> {
    unimplemented!()
}