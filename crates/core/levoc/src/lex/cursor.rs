use std::str::Chars;

pub struct Cursor<'a> {
    chars: Chars<'a>,
    remain: usize,

    #[cfg(debug_assertions)]
    prev: Option<char>,
}

impl<'a> Cursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars(),
            remain: text.len(),

            #[cfg(debug_assertions)]
            prev: None,
        }
    }
}

impl<'a> Cursor<'a> {
    pub fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }
}

impl Cursor<'_> {
    pub fn pos(&self) -> usize {
        self.remain - self.as_str().len()
    }

    pub fn rebase(&mut self) {
        self.remain = self.as_str().len();
    }

    pub(crate) fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next();
        #[cfg(debug_assertions)]
        {
            self.prev = ch;
        }

        ch
    }

    pub(crate) fn bump_nth(&mut self, n: usize) -> Option<char> {
        let ch = self.chars.nth(n);
        #[cfg(debug_assertions)]
        {
            self.prev = ch;
        }

        ch
    }

    pub(crate) fn bump_while(&mut self, pred: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&pred) {
            self.bump();
        }
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    pub(crate) fn peek_nth(&self, n: usize) -> Option<char> {
        self.chars.clone().nth(n)
    }

    pub(crate) fn prev(&self) -> Option<char> {
        #[cfg(debug_assertions)]
        {
            self.prev
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}
