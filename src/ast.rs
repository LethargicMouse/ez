use std::path::Path;

use crate::{fail::Fail, lex::Pos};

pub struct Module<'a> {
    pub source: Source<'a>,
    pub items: Vec<Item<'a>>,
}

pub enum Item<'a> {
    Fun(Fun<'a>),
    ExtFun(Header<'a>),
}

impl<'a> From<Fun<'a>> for Item<'a> {
    fn from(fun: Fun<'a>) -> Self {
        Self::Fun(fun)
    }
}

impl<'a> Item<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Item::Fun(fun) => fun.header.name,
            Item::ExtFun(header) => header.name,
        }
    }

    pub fn kind(&self) -> ItemKind {
        match self {
            Item::Fun(_) => ItemKind::Fun,
            Item::ExtFun(_) => ItemKind::ExtFun,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ItemKind {
    Fun,
    ExtFun,
}

pub struct Header<'a> {
    pub name: &'a str,
    pub ret_typ: Typ<'a>,
}

pub struct Fun<'a> {
    pub header: Header<'a>,
    pub body: Block<'a>,
}

pub struct Block<'a> {
    pub exprs: Vec<Expr<'a>>,
    pub out: Expr<'a>,
}

pub struct Call<'a> {
    pub name: &'a str,
    pub pos: Pos,
    pub args: Vec<Expr<'a>>,
}

pub struct Field<'a> {
    pub expr: Box<Expr<'a>>,
    pub name: &'a str,
}

pub enum Expr<'a> {
    Int(Int),
    Call(Call<'a>),
    Str(&'a str),
    Field(Field<'a>),
}

impl<'a> From<Call<'a>> for Expr<'a> {
    fn from(call: Call<'a>) -> Self {
        Self::Call(call)
    }
}

impl<'a> From<Int> for Expr<'a> {
    fn from(int: Int) -> Self {
        Self::Int(int)
    }
}

pub struct Int {
    pub val: u64,
    pub pos: Pos,
}

pub struct NameTyp<'a> {
    pub name: &'a str,
    pub pos: Pos,
}

pub enum Typ<'a> {
    Name(NameTyp<'a>),
    I32,
}

impl<'a> From<NameTyp<'a>> for Typ<'a> {
    fn from(typ: NameTyp<'a>) -> Self {
        Self::Name(typ)
    }
}
impl<'a> Typ<'a> {
    pub fn from_name(name: &'a str, pos: Pos) -> Typ {
        match name {
            "i32" => Self::I32,
            _ => NameTyp { name, pos }.into(),
        }
    }
}

pub struct Source<'a> {
    pub path: &'a Path,
    pub code: &'a str,
    pub lines: Vec<&'a str>,
}

impl<'a> Source<'a> {
    pub fn new(path: &'a Path, code: &'a str) -> Result<Self, Fail> {
        let lines = code.lines().collect();
        Ok(Self { code, path, lines })
    }
}
