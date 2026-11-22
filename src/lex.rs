use std::fmt::Display;

use crate::ast::Source;

pub struct Stream<'a> {
    pub source: Source<'a>,
    pos: Pos,
    cursor: usize,
}

impl<'a> Stream<'a> {
    pub fn new(source: Source<'a>) -> Self {
        Self {
            pos: Pos { line: 1, symbol: 1 },
            cursor: 0,
            source,
        }
    }

    pub fn next(&mut self) -> Token<'a> {
        self.advance_while(|c| c.is_ascii_whitespace());
        let mut chars = self.rest().iter();
        match chars.next() {
            Some(c) => match *c {
                b';' => self.give(Tok::Semi, 1),
                b'.' => self.give(Tok::Dot, 1),
                b',' => self.give(Tok::Comma, 1),
                b'(' => self.give(Tok::Parl, 1),
                b')' => self.give(Tok::Parr, 1),
                b'{' => self.give(Tok::Curl, 1),
                b'}' => self.give(Tok::Curr, 1),
                b'"' => {
                    self.cursor += 1;
                    let len = self.count_while(|c| c != b'"') + 2;
                    self.cursor -= 1;
                    let tok = if self.cursor + len == self.source.code.len() {
                        Tok::UnclosedStr
                    } else {
                        Tok::Str
                    };
                    self.give(tok, len)
                }
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

    fn give(&mut self, tok: Tok, len: usize) -> Token<'a> {
        let res = Token {
            body: &self.source.code[self.cursor..self.cursor + len],
            pos: self.pos,
            tok,
        };
        if false {
            eprintln!("give token {:?}", res);
        }
        self.advance(len);
        res
    }

    fn advance(&mut self, len: usize) {
        for c in &self.source.code.as_bytes()[self.cursor..][..len] {
            self.cursor += 1;
            if *c == b'\n' {
                self.pos.line += 1;
                self.pos.symbol = 1;
            } else {
                self.pos.symbol += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub tok: Tok,
    pub body: &'a str,
    pub pos: Pos,
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
    Str,
    UnclosedStr,
    Comma,
    Dot,
    Semi,
    Extern,
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
            Tok::Str => "<str>",
            Tok::UnclosedStr => "<unclosed str>",
            Tok::Comma => "`,`",
            Tok::Dot => "`.`",
            Tok::Semi => "`;`",
            Tok::Extern => "`extern`",
        }
    }
}

impl From<&'_ [u8]> for Tok {
    fn from(s: &'_ [u8]) -> Self {
        match s {
            b"fn" => Tok::Fun,
            b"extern" => Tok::Extern,
            _ => Tok::Name,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub line: u32,
    pub symbol: u32,
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.symbol)
    }
}
