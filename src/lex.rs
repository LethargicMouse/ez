use std::{fmt::Display, range::Range};

use crate::ast::Source;

pub type Span = Range<usize>;

pub struct Stream {
    pub source: Source,
    last: Option<Token>,
    loc: Loc,
    cursor: usize,
}

impl Stream {
    pub fn new(source: Source) -> Self {
        Self {
            loc: Loc {
                pos: Pos { line: 1, symbol: 1 },
                line: (0..0).into(),
            },
            last: None,
            cursor: 0,
            source,
        }
    }

    pub fn peek(&mut self) -> Token {
        let res = self.next();
        self.last = Some(res);
        res
    }

    pub fn next(&mut self) -> Token {
        if let Some(res) = self.last.take() {
            return res;
        }
        self.advance_while(|c| c.is_ascii_whitespace());
        let mut chars = self.rest().iter();
        match chars.next() {
            Some(c) => match *c {
                b'(' => self.give(Tok::Parl, 1),
                b')' => self.give(Tok::Parr, 1),
                b'{' => self.give(Tok::Curl, 1),
                b'}' => self.give(Tok::Curr, 1),
                c if c.is_ascii_alphabetic() || c == b'_' => {
                    let len = self.count_while(|c| c.is_ascii_alphanumeric() || c == b'_');
                    self.give(Tok::from(&self.rest()[..len]), len)
                }
                c if c.is_ascii_digit() => {
                    let len = self.count_while(|c| c.is_ascii_digit());
                    self.give(Tok::Int, len)
                }
                _ => self.give(Tok::Bad, 1),
            },
            None => self.give(Tok::Eof, 0),
        }
    }

    fn advance_while(&mut self, predicate: fn(u8) -> bool) {
        let len = self.count_while(predicate);
        self.advance(len);
    }

    fn count_while(&self, predicate: fn(u8) -> bool) -> usize {
        self.rest().iter().take_while(|c| predicate(**c)).count()
    }

    fn rest(&self) -> &[u8] {
        &self.source.code.as_bytes()[self.cursor..]
    }

    fn give(&mut self, tok: Tok, len: usize) -> Token {
        if false {
            eprintln!("tok {}", tok.as_str());
        }
        Token {
            span: (self.cursor..self.cursor + len).into(),
            loc: self.give_loc(len),
            tok,
        }
    }

    fn advance(&mut self, len: usize) {
        for c in &self.source.code.as_bytes()[self.cursor..][..len] {
            if *c == b'\n' {
                self.loc.next_line();
            } else {
                self.loc.next();
            }
        }
        self.cursor += len;
    }

    pub fn consume(&mut self, tok: Tok) {
        assert_eq!(tok, self.next().tok);
    }

    fn give_loc(&mut self, len: usize) -> Loc {
        let pos = self.loc.pos;
        self.advance(len);
        let line = self.loc.line;
        Loc { pos, line }
    }
}

#[derive(Clone, Copy)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    pub loc: Loc,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tok {
    Eof,
    Bad,
    Fun,
    Name,
    Parl,
    Parr,
    Curl,
    Curr,
    Int,
}

impl Tok {
    pub fn as_str(self) -> &'static str {
        match self {
            Tok::Eof => "<eof>",
            Tok::Bad => "<invalid>",
            Tok::Fun => "`fn`",
            Tok::Name => "<name>",
            Tok::Parl => "`(`",
            Tok::Parr => "`)`",
            Tok::Curl => "`{`",
            Tok::Curr => "`}`",
            Tok::Int => "<int>",
        }
    }
}

impl From<&'_ [u8]> for Tok {
    fn from(s: &'_ [u8]) -> Self {
        match s {
            b"fn" => Tok::Fun,
            _ => Tok::Name,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pos {
    pub line: u32,
    pub symbol: u32,
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.symbol)
    }
}

#[derive(Clone, Copy)]
pub struct Loc {
    pub pos: Pos,
    pub line: Span,
}
impl Loc {
    fn next_line(&mut self) {
        self.pos.line += 1;
        self.pos.symbol = 1;
        self.line.start = self.line.end + 1;
        self.line.end = self.line.start;
    }

    fn next(&mut self) {
        self.pos.symbol += 1;
        self.line.end += 1;
    }
}
