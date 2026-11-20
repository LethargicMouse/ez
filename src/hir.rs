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

pub struct Call {
    pub id: usize,
    pub typ: FunTyp,
}

pub enum Expr {
    Int(Typ, u64),
    Call(Call),
}

#[derive(Debug)]
pub struct FunTyp {
    params: Vec<Typ>,
    ret_typ_id: Box<Typ>,
}

#[derive(Debug)]
pub enum Typ {
    Int(IntTyp),
    Fun(FunTyp),
    Var(usize),

    Any,
    Err,
}

impl From<FunTyp> for Typ {
    fn from(fun: FunTyp) -> Self {
        Self::Fun(fun)
    }
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
            Typ::Any => false,
            Typ::Err => true,
            Typ::Var(_) => unreachable!("is_rigid nested typ var"),
            Typ::Fun(fun_typ) => fun_typ.params.iter().all(|t| t.is_rigid()),
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..) | Self::Err)
    }

    pub fn copy(&self) -> Self {
        match self {
            Typ::Int(typ) => Self::Int(*typ),
            Typ::Var(id) => Self::Var(*id),
            Typ::Any => Self::Any,
            Typ::Err => Self::Err,
            Typ::Fun(_) => unreachable!("copy of fun typ"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IntTyp {
    I32,
}
