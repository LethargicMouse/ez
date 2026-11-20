use std::{fs::read_to_string, path::PathBuf};

use crate::{
    fail::{self, Fail},
    lex::{Loc, Span},
    parse,
};

pub struct Module {
    pub source: Source,
    pub funs: Vec<Fun>,
}

impl Module {
    pub fn read(path: PathBuf) -> Result<Self, Fail> {
        parse::run(Source::read(path)?)
    }
}

pub struct Fun {
    pub name: Span,
    pub ret_typ: Option<Typ>,
    pub body: Block,
}

pub struct Block {
    pub out: Expr,
}

pub enum Expr {
    Int(Int),
}

impl From<Int> for Expr {
    fn from(int: Int) -> Self {
        Self::Int(int)
    }
}

pub struct Int {
    pub val: u64,
    pub loc: Loc,
}

pub enum Typ {
    Name(Span),
    I32,
}
impl Typ {
    pub fn from_span(span: Span, code: &str) -> Typ {
        match &code[span] {
            "i32" => Self::I32,
            _ => Self::Name(span),
        }
    }
}

pub struct Source {
    pub path: PathBuf,
    pub code: String,
}

impl Source {
    fn read(path: PathBuf) -> Result<Source, Fail> {
        let code = read_to_string(&path).map_err(|err| fail::file_read(&path, err))?;
        Ok(Self { code, path })
    }
}
