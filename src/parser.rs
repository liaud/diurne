#[derive(Debug, Copy, Clone)]
pub enum TokenKind {
    Ident,
    Digits,
    Colon,
    Star,
    Dot,
    Spaces,
}

#[derive(Debug, Copy, Clone)]
pub struct Span {
    column: u16,
    line: u16,
    start: u32,
    end: u32,
}

pub struct Token {
    kind: TokenKind,
    span: Span,
}

type SourceIter<'s> = std::iter::Peekable<std::str::CharIndices<'s>>

pub struct Tokenizer<'s> {
    source: SourceIter<'s>
    column: u16,
    line: u16,
}

impl<'s> Tokenizer<'s> {
    pub fn new(source: &'s str) -> Self {
        let iter = source.char_indices().peekable();

        Self {
            source,
            column: 0,
            line: 0,
        }
    }


    fn scan_spaces(&mut self, start: u32) -> Token {
        let span = self.scan_until(start, |c| !c.is_whitespace());
        self.make_token(span, TokenKind::Spaces)
    }

    fn scan_ident(&mut self, start: u32) -> Token {
        let span = self.scan_until(start, |c| !Self::is_ident(c));
        self.make_token(span, TokenKind::Ident)
    }

    fn scan_digits(&mut self, start: u32) -> Token {
        let span = self.scan_until(start, |c| !c.is_digit());
        self.make_token(span, TokenKind::Digits)
    }

    fn scan_until(&self, start: u32, until: impl Fn(char) -> bool) -> std::ops::Range<u32> {
        let mut last = start;
        while let Some((idx, c)) = self.peek() {
            if until(c) {
                break;
            }

            last = idx;
            self.bump();
        }

        start..(last + 1)
    }

    fn peek(&self) -> Option<char> {
        self.source.peek()
    }

    fn bump(&self) -> Option<char> {
        let c = self.source.next();
        if c.is_none() {
            return None;
        }
        self.column += 1;

        if Self::is_break(c) => {
            if c == '\r' => {
                assert_eq!(Some('\n'), self.source.next());
            }
                
            self.line += 1;
            self.column = 0;
        }

        Some(c)
    }
    
    fn make_token(&self, range: std::ops::Range<u32>, kind: TokenKind) -> Token {
        Token {
            span: Span {
                column: self.column,
                line: self.line,
                start: range.start,
                end: range.end,
            }
        }
    }

    fn is_break(c: char) -> bool {
        match c {
            '\n' => true,
            '\r' => true,
            '\u{2028}' => true,
            '\u{2029}' => true,
            _ => false
        }
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        while let Some((start, c)) in self.peek() {
            let start = start as u32;

            let token = match c {
                ':' => {
                    let token = self.make_token(start..(start+1), TokenKind::Colon);
                    self.bump();
                    token
                },
                '*' => {
                    let token = self.make_token(start..(start+1), TokenKind::Star);
                    self.bump();
                    token
                },
                '.' => {
                    let token = self.make_token(start..(start+1), TokenKind::Dot);
                    self.bump();
                    token
                },
                c if c.is_whitespace() => self.scan_spaces(start),
                c if c.is_digit() => self.scan_digits(start),
                _ => self.scan_ident(start),
            }

            return Some(token);
        }

        None
    }
}