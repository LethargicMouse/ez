use std::fmt::Display;

use crate::{hir, lir};

pub struct Program {
    pub funs: Vec<Fun>,
}

pub struct Fun {
    pub ret_typ: Typ,
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub out: Out,
}

pub enum Out {
    Ret(Expr),
}

pub enum Expr {
    Int(IntTyp, u64),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            lir::Expr::Int(typ, int) => write!(f, "{} {}", typ, int),
        }
    }
}

#[derive(Debug)]
pub enum Typ {
    Int(IntTyp),
}

impl Typ {
    pub fn to_int(self) -> IntTyp {
        match self {
            Typ::Int(typ) => typ,
            // _ => unreachable!("{:?}.to_int()", self),
        }
    }
}

impl Display for Typ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Typ::Int(typ) => write!(f, "{}", typ),
        }
    }
}

#[derive(Debug)]
pub enum IntTyp {
    I32,
}

impl From<hir::IntTyp> for IntTyp {
    fn from(typ: hir::IntTyp) -> Self {
        match typ {
            hir::IntTyp::I32 => lir::IntTyp::I32,
        }
    }
}

impl Display for IntTyp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntTyp::I32 => write!(f, "i32"),
        }
    }
}
