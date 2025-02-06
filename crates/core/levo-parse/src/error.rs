use levo_lex::Op;

#[derive(Debug, Clone)]
pub enum ParseError {
    OpInPrim(Op),
    BadBinOp,
    BadPrim,
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpInPrim(_) => write!(f, "operator in primitive expression"),
            Self::BadBinOp => write!(f, "bad binary operation"),
            Self::BadPrim => write!(f, "bad primary expression"),
        }
    }
}
