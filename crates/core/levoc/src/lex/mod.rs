use unicode_ident::{is_xid_continue, is_xid_start};

use self::cursor::Cursor;
use self::token::{
    Base::*,
    CommentKind,
    Delim::*,
    LitKind::{self, *},
    Punc::*,
    Token,
    TokenKind::{self, *},
};

pub mod cursor;
pub mod token;

#[cfg(test)]
mod tests;

trait CharExt {
    fn is_newline(self) -> bool;

    fn is_ident_start(self) -> bool;
    fn is_ident_continue(self) -> bool;
}

impl CharExt for char {
    fn is_newline(self) -> bool {
        matches!(
            self,
            '\n' | '\r' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
        )
    }

    fn is_ident_start(self) -> bool {
        self == '_' || is_xid_start(self)
    }

    fn is_ident_continue(self) -> bool {
        is_xid_continue(self)
    }
}

impl Cursor<'_> {
    pub fn next_token(&mut self) -> Option<Token> {
        let kind = match self.bump()? {
            ch if ch.is_whitespace() => self.eat_whitespace(ch),
            ch if ch.is_ident_start() => self.eat_ident(),
            ch if ch.is_ascii_digit() => {
                let kind = self.eat_num_lit(ch);
                let suffix_start = self.pos() as u32;
                self.eat_suffix();
                Lit { kind, suffix_start }
            }

            '\'' => {
                let terminated = self.eat_char_lit();
                let suffix_start = self.pos() as u32;
                self.eat_suffix();
                Lit {
                    kind: Char { terminated },
                    suffix_start,
                }
            }
            '"' => {
                let terminated = self.eat_str_lit();
                let suffix_start = self.pos() as u32;
                self.eat_suffix();
                Lit {
                    kind: Str { terminated },
                    suffix_start,
                }
            }

            '/' => match self.peek() {
                Some('/') => self.eat_line_comment(),
                Some('*') => self.eat_block_comment(),
                _ => Punc(Slash),
            },

            '+' => Punc(Plus),
            '-' => Punc(Minus),
            '*' => Punc(Asterisk),
            '%' => Punc(Percent),

            '&' => Punc(Amp),
            '|' => Punc(Bar),
            '^' => Punc(Caret),
            '!' => Punc(Bang),

            '=' => Punc(Eq),
            '<' => Punc(Lt),
            '>' => Punc(Gt),

            '.' => Punc(Dot),
            ',' => Punc(Comma),
            ':' => Punc(Colon),
            ';' => Punc(Semi),

            '(' => Open(Paren),
            '[' => Open(Brack),
            '{' => Open(Brace),
            ')' => Close(Paren),
            ']' => Close(Brack),
            '}' => Close(Brace),

            _ => Unknown,
        };

        let token = Token::new(self.pos(), kind);
        self.rebase();
        Some(token)
    }

    fn eat_whitespace(&mut self, first: char) -> TokenKind {
        debug_assert!(self.prev().is_some_and(|ch| ch.is_whitespace()));
        match first {
            '\r' => {
                if self.peek().is_some_and(|ch| ch == '\n') {
                    _ = self.bump();
                }
                Newline
            }
            ch if ch.is_newline() => Newline,
            _ => {
                self.bump_while(|ch| ch.is_whitespace());
                Whitespace
            }
        }
    }

    fn eat_line_comment(&mut self) -> TokenKind {
        debug_assert!(
            self.prev().is_some_and(|ch| ch == '/') && self.peek().is_some_and(|ch| ch == '/')
        );

        _ = self.bump();
        let kind = match self.peek() {
            Some('/') => {
                _ = self.bump();
                CommentKind::OuterDoc
            }
            Some('!') => {
                _ = self.bump();
                CommentKind::InnerDoc
            }
            _ => CommentKind::Normal,
        };

        self.bump_while(|ch| ch != '\n');
        LineComment { kind }
    }

    fn eat_block_comment(&mut self) -> TokenKind {
        debug_assert!(
            self.prev().is_some_and(|ch| ch == '/') && self.peek().is_some_and(|ch| ch == '*')
        );

        _ = self.bump();
        let kind = match self.peek() {
            Some('*') => {
                _ = self.bump();
                if self.peek().is_some_and(|ch| ch == '/') {
                    _ = self.bump();
                    return BlockComment {
                        kind: CommentKind::Normal,
                        terminated: true,
                    };
                } else {
                    CommentKind::OuterDoc
                }
            }
            Some('!') => {
                _ = self.bump();
                CommentKind::InnerDoc
            }
            _ => CommentKind::Normal,
        };

        let mut depth: u32 = 1;
        loop {
            self.bump_while(|ch| !matches!(ch, '*' | '/'));
            match (self.bump(), self.peek()) {
                (None, _) | (_, None) => break,
                (Some('/'), Some('*')) => {
                    _ = self.bump();
                    depth += 1;
                }
                (Some('*'), Some('/')) => {
                    _ = self.bump();
                    depth -= 1;
                    if depth <= 0 {
                        break;
                    }
                }
                _ => {}
            }
        }

        BlockComment {
            kind,
            terminated: depth <= 0,
        }
    }

    fn eat_ident(&mut self) -> TokenKind {
        debug_assert!(self.prev().is_some_and(|ch| ch.is_ident_start()));
        self.bump_while(|ch| ch.is_ident_continue());
        Ident
    }

    fn eat_num_lit(&mut self, first: char) -> LitKind {
        debug_assert!(self.prev().is_some_and(|ch| ch.is_ascii_digit()));

        let mut base = Decimal;
        if first == '0' {
            match self.peek() {
                Some('b') => {
                    _ = self.bump();
                    base = Binary;
                    if !self.eat_decimal_digits() {
                        return Int { base, empty: true };
                    }
                }
                Some('o') => {
                    _ = self.bump();
                    base = Octal;
                    if !self.eat_decimal_digits() {
                        return Int { base, empty: true };
                    }
                }
                Some('x') => {
                    _ = self.bump();
                    base = Hexadecimal;
                    if !self.eat_hexadecimal_digits() {
                        return Int { base, empty: true };
                    }
                }
                Some('0'..='9' | '_') => _ = self.eat_decimal_digits(),
                Some('.' | 'e' | 'E') => {}
                _ => return Int { base, empty: false },
            }
        } else {
            self.eat_decimal_digits();
        }

        match self.peek() {
            Some('.')
                if !self
                    .peek_nth(1)
                    .is_none_or(|ch| ch == '.' || ch.is_ident_start()) =>
            {
                _ = self.bump();
                let mut exp_empty = false;
                if self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                    self.eat_decimal_digits();
                    match self.peek() {
                        Some('e' | 'E') => {
                            _ = self.bump();
                            exp_empty = !self.eat_float_exponent();
                        }
                        _ => {}
                    }
                }

                Float { base, exp_empty }
            }
            Some('e' | 'E') => {
                _ = self.bump();
                let exp_empty = !self.eat_float_exponent();
                Float { base, exp_empty }
            }

            _ => Int { base, empty: false },
        }
    }

    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        while let Some(ch) = self.peek() {
            match ch {
                '_' => _ = self.bump(),
                '0'..='9' => {
                    has_digits = true;
                    _ = self.bump();
                }

                _ => break,
            }
        }

        has_digits
    }

    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        while let Some(ch) = self.peek() {
            match ch {
                '_' => _ = self.bump(),
                '0'..='9' | 'a'..='f' | 'A'..='F' => {
                    has_digits = true;
                    _ = self.bump();
                }

                _ => break,
            }
        }

        has_digits
    }

    fn eat_float_exponent(&mut self) -> bool {
        debug_assert!(matches!(self.prev(), Some('e' | 'E')));

        if matches!(self.peek(), Some('-' | '+')) {
            self.bump();
        }

        self.eat_decimal_digits()
    }

    fn eat_char_lit(&mut self) -> bool {
        debug_assert!(matches!(self.prev(), Some('\'')));

        if self.peek_nth(1) == Some('\'') && self.peek() != Some('\\') {
            self.bump_nth(1);
            return true;
        }

        loop {
            break if let Some(ch) = self.bump() {
                match ch {
                    '\\' => {
                        // skip the next character
                        _ = self.bump();
                        continue;
                    }
                    '\n' => false,
                    '\'' => true,
                    _ => continue,
                }
            } else {
                false
            };
        }
    }

    fn eat_str_lit(&mut self) -> bool {
        debug_assert!(matches!(self.prev(), Some('"')));

        loop {
            break if let Some(ch) = self.bump() {
                match ch {
                    '\\' => {
                        // skip the next character
                        _ = self.bump();
                        continue;
                    }
                    '\n' => false,
                    '"' => true,
                    _ => continue,
                }
            } else {
                false
            };
        }
    }

    fn eat_suffix(&mut self) {
        if self.peek().is_some_and(char::is_ident_start) {
            _ = self.bump();
            self.bump_while(char::is_ident_continue);
        }
    }
}
