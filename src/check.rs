use std::collections::HashMap;

use crate::{
    ast::{self, Source},
    fail::{Errors, Fail},
    hir,
    lex::Pos,
    util::Counter,
};

pub fn run(module: ast::Module) -> Result<hir::Program, Fail> {
    let mut info = ProgramInfo::new();
    let module = module.check(&mut info);
    info.modules.push(module);
    let main_id = get_main_id(&mut info)?;
    if info.errors.present() {
        Err(Fail)
    } else {
        Ok(hir::Program {
            modules: info.modules,
            main_id,
        })
    }
}

impl<'a> ast::Module<'a> {
    fn check(self, program: &mut ProgramInfo<'a>) -> hir::Module<'a> {
        let mut info = ModuleInfo {
            items: HashMap::new(),
            source: self.source,
        };
        for (index, fun) in self.funs.iter().enumerate() {
            info.items.insert(fun.name, index);
        }
        let funs = self
            .funs
            .into_iter()
            .map(|f| f.check(program, &info))
            .collect();
        hir::Module {
            path: info.source.path,
            items: info.items,
            funs,
        }
    }
}

impl<'a> ast::Fun<'a> {
    fn check(self, program: &mut ProgramInfo<'a>, module: &ModuleInfo) -> hir::Fun {
        let mut info = FunInfo::new();
        let ret_typ = self.ret_typ.check(program, module);
        let expr = self.body.check(&ret_typ, program, module, &mut info);
        info.blocks.push(hir::Block {
            out: hir::Out::Ret(expr),
        });
        check_infers(program, module, &info);
        hir::Fun {
            typ_vars: info.typ_vars,
            blocks: info.blocks,
            ret_typ,
        }
    }
}

impl<'a> ast::Typ<'a> {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo) -> hir::Typ {
        match self {
            ast::Typ::Name(typ) => {
                program
                    .errors
                    .not_declared(&module.source, typ.pos, typ.name);
                hir::Typ::Err
            }
            ast::Typ::I32 => hir::IntTyp::I32.into(),
        }
    }
}

impl<'a> ast::Block<'a> {
    fn check(
        self,
        hint: &hir::Typ,
        program: &mut ProgramInfo<'a>,
        module: &ModuleInfo,
        fun: &mut FunInfo,
    ) -> hir::Expr {
        for expr in self.exprs {
            let expr = expr.check(&hir::Typ::Any, program, module, fun);
        }
        self.out.check(&hint, program, module, fun)
    }
}

impl<'a> ast::Expr<'a> {
    fn check(
        self,
        hint: &hir::Typ,
        program: &mut ProgramInfo<'a>,
        module: &ModuleInfo,
        fun: &mut FunInfo,
    ) -> hir::Expr {
        match self {
            ast::Expr::Int(int) => int.check(hint, fun),
            ast::Expr::Call(call) => call.check(program, module, fun),
            ast::Expr::Str(span) => todo!(),
            ast::Expr::Field(field) => todo!(),
        }
    }
}

impl<'a> ast::Call<'a> {
    fn check(
        self,
        program: &mut ProgramInfo<'a>,
        module: &ModuleInfo,
        fun: &mut FunInfo,
    ) -> hir::Expr {
        let Some(&index) = module.items.get(self.name) else {
            program
                .errors
                .not_declared(&module.source, self.pos, self.name);
            for arg in self.args {
                arg.check(&hir::Typ::Err, program, module, fun);
            }
            return hir::Expr::Void;
        };
        todo!()
    }
}

impl ast::Int {
    fn check(self, hint: &hir::Typ, fun: &mut FunInfo) -> hir::Expr {
        let mut typ = if hint.is_int() {
            hint.copy()
        } else {
            hir::Typ::Any
        };
        if !typ.is_rigid() {
            let id = fun.typ_vars.len();
            fun.typ_vars.push(typ);
            typ = hir::Typ::Var(id);
            fun.infer_checks.push(InferCheck { pos: self.pos, id });
        }
        hir::Expr::Int(typ, self.val)
    }
}

fn get_main_id(ctx: &mut ProgramInfo) -> Result<hir::FunID, Fail> {
    let mod_id = 0;
    match ctx.modules[0].items.get("main") {
        Some(&method_id) => Ok(hir::FunID { mod_id, method_id }),
        None => Err(ctx.errors.no_main(&ctx.modules[0].path)),
    }
}

struct ModuleInfo<'a> {
    items: HashMap<&'a str, usize>,
    source: Source<'a>,
}

struct InferCheck {
    id: usize,
    pos: Pos,
}

struct FunInfo {
    blocks: Vec<hir::Block>,
    typ_vars: Vec<hir::Typ>,
    infer_checks: Vec<InferCheck>,
}

impl FunInfo {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            typ_vars: Vec::new(),
            infer_checks: Vec::new(),
        }
    }
}

fn check_infers(program: &mut ProgramInfo, module: &ModuleInfo, fun: &FunInfo) {
    for check in &fun.infer_checks {
        if !fun.typ_vars[check.id].is_rigid() {
            program.errors.cannot_infer(&module.source, check.pos);
        }
    }
}

struct ProgramInfo<'a> {
    errors: Errors,
    modules: Vec<hir::Module<'a>>,
}

impl<'a> ProgramInfo<'a> {
    fn new() -> Self {
        Self {
            errors: Errors::new(),
            modules: Vec::new(),
        }
    }
}
