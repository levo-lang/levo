use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Token {
    pub len: usize,
    pub kind: TokenKind,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    Unknown,

    Ident,
    Num,

    Whitespace,

    Op(Op),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Plus,     // +
    Minus,    // -
    Asterisk, // *
    Slash,    // /
    Percent,  // %
}

pub struct Lexer<'a> {
    remain: usize,
    chars: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            remain: text.len(),
            chars: text.chars(),
        }
    }

    pub fn lex(&mut self) -> Option<Token> {
        let cur = self.remain;
        let kind = match self.peek()? {
            '+' => self.lex_op(Op::Plus),
            '-' => self.lex_op(Op::Minus),
            '*' => self.lex_op(Op::Asterisk),
            '%' => self.lex_op(Op::Percent),
            '/' => {
                _ = self.advance();
                match self.peek() {
                    _ => self.lex_op(Op::Slash),
                }
            }

            ch if unicode_ident::is_xid_start(ch) => self.lex_ident(),
            ch if ch.is_ascii_digit() => self.lex_num(),
            ch if ch.is_whitespace() => self.lex_whitespace(),

            _ => self.lex_single(TokenKind::Unknown),
        };

        Some(Token {
            len: cur - self.remain,
            kind,
        })
    }

    fn lex_ident(&mut self) -> TokenKind {
        self.lex_while(TokenKind::Ident, unicode_ident::is_xid_continue)
    }

    fn lex_num(&mut self) -> TokenKind {
        self.lex_while(TokenKind::Num, |ch| ch.is_ascii_digit())
    }

    fn lex_whitespace(&mut self) -> TokenKind {
        self.lex_while(TokenKind::Whitespace, char::is_whitespace)
    }

    fn lex_op(&mut self, op: Op) -> TokenKind {
        self.lex_single(TokenKind::Op(op))
    }

    fn lex_single(&mut self, kind: TokenKind) -> TokenKind {
        _ = self.advance();
        kind
    }

    fn lex_while<F>(&mut self, kind: TokenKind, pred: F) -> TokenKind
    where
        F: Fn(char) -> bool,
    {
        _ = self.advance();
        while let Some(ch) = self.peek() {
            if pred(ch) {
                _ = self.advance();
            } else {
                break;
            }
        }

        kind
    }
}

impl Lexer<'_> {
    fn advance(&mut self) -> Option<char> {
        self.chars.next().inspect(|ch| self.remain -= ch.len_utf8())
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }
}

#[test]
fn lexer_test() {
    let texts = ["a + b + c"];
    for (num, text) in texts.iter().enumerate() {
        println!("====== {num} ======");
        let mut lexer = Lexer::new(text);
        std::iter::from_fn(|| lexer.lex()).for_each(|token| println!("{token:?}"));
    }
}
