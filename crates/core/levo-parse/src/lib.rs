mod error;

use levo_lex::{Lexer, Op, Token, TokenKind};

pub use self::error::ParseError;

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(Ident),
    Num(Num),
    Bin(BinExpr),
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Num {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BinExpr {
    pub op: BinOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl BinExpr {
    pub fn new(op: BinOp, left: Expr, right: Expr) -> Self {
        Self {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, // a + b
    Sub, // a - b
    Mul, // a * b
    Div, // a / b
    Mod, // a % b
}

impl BinOp {
    pub fn prec(&self) -> Prec {
        match self {
            Self::Add | Self::Sub => Prec::Add,
            Self::Mul | Self::Div | Self::Mod => Prec::Mul,
        }
    }

    pub fn assoc(&self) -> Assoc {
        match self {
            _ => Assoc::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prec {
    Min,

    Add,
    Mul,

    Max,
}

impl Prec {
    pub const fn next(&self) -> Self {
        match self {
            Self::Min => Self::Add,

            Self::Add => Self::Mul,
            Self::Mul => Self::Max,

            Self::Max => Self::Max,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Assoc {
    Left,
    Right,
}

pub struct Parser<'a> {
    text: &'a str,
    cur: usize,

    lexer: Lexer<'a>,
    token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            cur: 0,

            token: None,
            lexer: Lexer::new(text),
        }
    }

    pub fn parse_expr(&mut self) -> Result<Option<Expr>, ParseError> {
        let left = self.parse_prim_expr()?;
        self.parse_bin_expr(left, Prec::Min)
    }

    pub fn parse_bin_expr(
        &mut self,
        left: Option<Expr>,
        min: Prec,
    ) -> Result<Option<Expr>, ParseError> {
        Ok(if let Some(mut left) = left {
            let mut token = self.next_proper_token();
            while let Some(op) = self.get_bin_op(token.as_ref())? {
                let prec = op.prec();
                if prec < min {
                    break;
                }

                let right = self.parse_prim_expr()?;
                let Some(right) = self.parse_bin_expr(
                    right,
                    match op.assoc() {
                        Assoc::Left => prec.next(),
                        Assoc::Right => prec,
                    },
                )?
                else {
                    return Err(ParseError::BadBinOp);
                };

                left = Expr::Bin(BinExpr::new(op, left, right));
                token = self.next_proper_token();
            }

            self.shelf_token(token);
            Some(left)
        } else {
            None
        })
    }

    fn get_bin_op(&self, token: Option<&Token>) -> Result<Option<BinOp>, ParseError> {
        Ok(match token {
            Some(token) => match token.kind {
                TokenKind::Op(op) => Some(match op {
                    Op::Plus => BinOp::Add,
                    Op::Minus => BinOp::Sub,
                    Op::Asterisk => BinOp::Mul,
                    Op::Slash => BinOp::Div,
                    Op::Percent => BinOp::Mod,
                }),
                _ => None,
            },
            None => None,
        })
    }

    fn parse_prim_expr(&mut self) -> Result<Option<Expr>, ParseError> {
        let start = self.cur;
        Ok(match self.next_proper_token() {
            Some(token) => Some(match token.kind.clone() {
                TokenKind::Ident => {
                    let end = self.cur;
                    Expr::Ident(Ident {
                        name: self.text[start..end].to_string(),
                    })
                }
                TokenKind::Num => {
                    let end = self.cur;
                    Expr::Num(Num {
                        value: self.text[start..end].to_string(),
                    })
                }
                _ => {
                    self.shelf_token(Some(token));
                    return Err(ParseError::BadPrim);
                }
            }),
            None => None,
        })
    }
}

impl Parser<'_> {
    fn next_token(&mut self) -> Option<Token> {
        self.token
            .take()
            .or_else(|| self.lexer.lex())
            .inspect(|token| self.cur += token.len)
    }

    fn next_proper_token(&mut self) -> Option<Token> {
        std::iter::from_fn(|| self.next_token())
            .skip_while(|token| matches!(token.kind, TokenKind::Whitespace))
            .next()
    }

    fn shelf_token(&mut self, token: Option<Token>) {
        self.token = token.inspect(|token| self.cur -= token.len);
    }
}

#[test]
fn parser_test() -> Result<(), ParseError> {
    let texts = [
        "a + b * c",
        "a * b + c",
        "a + b + c",
        "a - b + c",
        "a - b - c",
        "a * b * c - d + e / f",
    ];
    for (num, text) in texts.iter().enumerate() {
        println!("====== {num} ======");
        let mut parser = Parser::new(text);
        std::iter::from_fn(|| parser.parse_expr().transpose())
            .for_each(|token| println!("{token:?}"));
    }

    Ok(())
}
