#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Normal,
    OuterDoc,
    InnerDoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delim {
    Paren, // ( )
    Brack, // [ ]
    Brace, // { }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    Binary = 2,
    Octal = 4,
    Decimal = 10,
    Hexadecimal = 16,
}

#[derive(Debug, Clone, Copy)]
pub enum LitKind {
    Int { base: Base, empty: bool },
    Float { base: Base, exp_empty: bool },
    Char { terminated: bool },
    Str { terminated: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punc {
    // arithmetic
    Plus,     // +
    Minus,    // -
    Asterisk, // *
    Slash,    // /
    Percent,  // %

    // logic
    Amp,   // &
    Bar,   // |
    Caret, // ^
    Bang,  // !

    // comparison
    Eq, // =
    Lt, // <
    Gt, // >

    // punctuation
    Dot,   // .
    Comma, // ,
    Colon, // :
    Semi,  // ;
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub len: usize,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy)]
pub enum TokenKind {
    Unknown,

    Whitespace,
    Newline,

    LineComment {
        style: CommentKind,
    },
    BlockComment {
        style: CommentKind,
        terminated: bool,
    },

    Ident,
    Lit {
        kind: LitKind,
        suffix_start: u32,
    },

    Punc(Punc),

    // delimiter
    Open(Delim),
    Close(Delim),
}
