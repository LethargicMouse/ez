use std::collections::HashMap;

use crate::{
    ast::{self, Source},
    fail::{Errors, Fail},
    hir,
    lex::Loc,
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

impl ast::Module {
    fn check(self, program: &mut ProgramInfo) -> hir::Module {
        let mut info = ModuleInfo {
            items: HashMap::new(),
            source: self.source,
        };
        for (index, fun) in self.funs.iter().enumerate() {
            let id = program.id_map.get(info.source.code[fun.name].into());
            info.items.insert(id, index);
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

impl ast::Fun {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo) -> hir::Fun {
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

impl ast::Typ {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo) -> hir::Typ {
        match self {
            ast::Typ::Name(name, loc) => {
                program.errors.not_declared(&module.source, loc, name);
                hir::Typ::Err
            }
            ast::Typ::I32 => hir::IntTyp::I32.into(),
        }
    }
}

impl ast::Block {
    fn check(
        self,
        hint: &hir::Typ,
        program: &mut ProgramInfo,
        module: &ModuleInfo,
        fun: &mut FunInfo,
    ) -> hir::Expr {
        for expr in self.exprs {
            let expr = expr.check(&hir::Typ::Any, program, module, fun);
        }
        self.out.check(&hint, program, module, fun)
    }
}

impl ast::Expr {
    fn check(
        self,
        hint: &hir::Typ,
        program: &mut ProgramInfo,
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

impl ast::Call {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo, fun: &mut FunInfo) -> hir::Expr {
        let id = program.id_map.get(module.source.code[self.name].into());
        let Some(&index) = module.items.get(&id) else {
            program
                .errors
                .not_declared(&module.source, self.loc, self.name);
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
            fun.infer_checks.push(InferCheck { loc: self.loc, id });
        }
        hir::Expr::Int(typ, self.val)
    }
}

fn get_main_id(ctx: &mut ProgramInfo) -> Result<hir::FunID, Fail> {
    let mod_id = 0;
    match ctx.modules[0].items.get(&ctx.id_map.get("main".into())) {
        Some(&method_id) => Ok(hir::FunID { mod_id, method_id }),
        None => Err(ctx.errors.no_main(&ctx.modules[0].path)),
    }
}

struct IdMap {
    counter: Counter,
    map: HashMap<String, u32>,
}

impl IdMap {
    fn new() -> Self {
        Self {
            counter: Counter::new(),
            map: HashMap::new(),
        }
    }

    fn get(&mut self, name: String) -> u32 {
        *self.map.entry(name).or_insert(self.counter.next())
    }
}

struct ModuleInfo {
    items: HashMap<u32, usize>,
    source: Source,
}

struct InferCheck {
    id: usize,
    loc: Loc,
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
            program.errors.cannot_infer(&module.source, check.loc);
        }
    }
}

struct ProgramInfo {
    id_map: IdMap,
    errors: Errors,
    modules: Vec<hir::Module>,
}

impl ProgramInfo {
    fn new() -> Self {
        Self {
            id_map: IdMap::new(),
            errors: Errors::new(),
            modules: Vec::new(),
        }
    }
}
