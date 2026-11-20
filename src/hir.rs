use std::{collections::HashMap, path::PathBuf};

pub struct Program {
    pub modules: Vec<Module>,
    pub main_id: FunID,
}

pub struct FunID {
    pub mod_id: usize,
    pub method_id: usize,
}

pub struct Module {
    pub path: PathBuf,
    pub funs: Vec<Fun>,
    pub items: HashMap<u32, usize>,
}

pub struct Fun {
    pub typ_vars: Vec<Typ>,
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
    Int(Typ, u64),
}

impl Expr {
    pub fn typ(&self) -> Typ {
        match self {
            Expr::Int(typ, _) => *typ,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Typ {
    Int(IntTyp),
    Var(usize),
    Any,
    Err,
}

impl From<IntTyp> for Typ {
    fn from(typ: IntTyp) -> Self {
        Self::Int(typ)
    }
}

impl Typ {
    pub fn is_rigid(&self) -> bool {
        match self {
            Typ::Int(_) => true,
            Typ::Var(_) => unreachable!("is_rigid nested typ var"),
            Typ::Any => false,
            Typ::Err => true,
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IntTyp {
    I32,
}
