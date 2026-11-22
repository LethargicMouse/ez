use std::collections::HashMap;

use crate::{
    ast::{self, ItemKind, Source},
    fail::{Errors, Fail},
    hir,
    lex::Pos,
};

pub fn run(module: ast::Module) -> Result<hir::Program, Fail> {
    let mut info = ProgramInfo::new();
    let module_info = module.check(&mut info);
    let main_id = get_main_id(&mut info, &module_info)?;
    info.modules.push(module_info.into());
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
    fn check(self, program: &mut ProgramInfo) -> ModuleInfo<'a> {
        let mut info = ModuleInfo::new(self.source);
        for (index, item) in self.items.iter().enumerate() {
            info.item_ids.insert(item.name(), (item.kind(), index));
        }
        for item in self.items {
            item.check(program, &mut info);
        }
        info
    }
}

impl<'a> ast::Item<'a> {
    fn check(self, program: &mut ProgramInfo, module: &mut ModuleInfo) {
        match self {
            ast::Item::Fun(fun) => module.funs.push(fun.check(program, module)),
            ast::Item::ExtFun(header) => module.ext_funs.push(header.check(program, module)),
        }
    }
}

impl<'a> ast::Fun<'a> {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo) -> hir::Fun {
        let mut info = FunInfo::new();
        let header = self.header.check(program, module);
        let expr = self.body.check(&header.ret_typ, program, module, &mut info);
        info.blocks.push(hir::Block {
            out: hir::Out::Ret(expr),
        });
        check_infers(program, module, &info);
        hir::Fun {
            typ_vars: info.typ_vars,
            blocks: info.blocks,
            header,
        }
    }
}

impl<'a> ast::Header<'a> {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo) -> hir::Header {
        hir::Header {
            ret_typ: self.ret_typ.check(program, module),
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

impl<'a> ast::Expr<'a> {
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

impl<'a> ast::Call<'a> {
    fn check(self, program: &mut ProgramInfo, module: &ModuleInfo, fun: &mut FunInfo) -> hir::Expr {
        let Some(&index) = module.item_ids.get(self.name) else {
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

fn get_main_id(program: &mut ProgramInfo, module: &ModuleInfo) -> Result<hir::FunID, Fail> {
    let mod_id = 0;
    match module.item_ids.get("main") {
        Some(&(kind, method_id)) => match kind {
            ItemKind::Fun => Ok(hir::FunID { mod_id, method_id }),
            _ => Err(program
                .errors
                .main_not_fun(&module.source, item_pos(kind, method_id, module))),
        },
        None => Err(program.errors.no_main(&module.source.path)),
    }
}

fn item_pos(kind: ItemKind, id: usize, module: &ModuleInfo) -> Pos {
    match kind {
        ItemKind::Fun => module.fun_poses[id],
        ItemKind::ExtFun => todo!(),
    }
}

struct ModuleInfo<'a> {
    item_ids: HashMap<&'a str, (ItemKind, usize)>,
    funs: Vec<hir::Fun>,
    fun_poses: Vec<Pos>,
    ext_funs: Vec<hir::Header>,
    source: Source<'a>,
}

impl<'a> ModuleInfo<'a> {
    fn new(source: Source<'a>) -> Self {
        ModuleInfo {
            item_ids: HashMap::new(),
            funs: Vec::new(),
            ext_funs: Vec::new(),
            fun_poses: Vec::new(),
            source,
        }
    }
}

impl<'a> From<ModuleInfo<'a>> for hir::Module {
    fn from(info: ModuleInfo<'a>) -> Self {
        Self {
            ext_funs: info.ext_funs,
            funs: info.funs,
        }
    }
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

struct ProgramInfo {
    errors: Errors,
    modules: Vec<hir::Module>,
}

impl ProgramInfo {
    fn new() -> Self {
        Self {
            errors: Errors::new(),
            modules: Vec::new(),
        }
    }
}
