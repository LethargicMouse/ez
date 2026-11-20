use crate::{hir, lir};

pub fn run(program: hir::Program) -> lir::Program {
    let mut funs = Vec::new();
    program.modules[program.main_id.mod_id].funs[program.main_id.method_id].mono(&mut funs);
    lir::Program { funs }
}

impl hir::Fun {
    fn mono(&self, funs: &mut Vec<lir::Fun>) {
        funs.push(lir::Fun {
            blocks: self.blocks.iter().map(|b| b.mono(&self.typ_vars)).collect(),
            ret_typ: self.ret_typ.mono(&self.typ_vars),
        });
    }
}

impl hir::Block {
    fn mono(&self, typ_vars: &[hir::Typ]) -> lir::Block {
        lir::Block {
            out: self.out.mono(typ_vars),
        }
    }
}

impl hir::Out {
    fn mono(&self, typ_vars: &[hir::Typ]) -> lir::Out {
        match self {
            hir::Out::Ret(expr) => lir::Out::Ret(expr.mono(typ_vars)),
        }
    }
}

impl hir::Expr {
    fn mono(&self, typ_vars: &[hir::Typ]) -> lir::Expr {
        match self {
            hir::Expr::Int(typ, int) => lir::Expr::Int(typ.mono(typ_vars).to_int(), *int),
            hir::Expr::Call(call) => todo!(),
        }
    }
}

impl hir::Typ {
    fn mono(&self, typ_vars: &[hir::Typ]) -> lir::Typ {
        match self {
            hir::Typ::Int(typ) => lir::IntTyp::from(*typ).into(),
            hir::Typ::Var(id) => typ_vars[*id].mono(typ_vars),
            hir::Typ::Any => unreachable!("mono any typ"),
            hir::Typ::Err => unreachable!("mono err typ"),
            hir::Typ::Fun(typ) => todo!(),
        }
    }
}
