use std::collections::HashMap;

use crate::{
    ast::{self, Source},
    fail::{Errors, Fail},
    hir,
    lex::Loc,
    util::Counter,
};

pub fn run(module: ast::Module) -> Result<hir::Program, Fail> {
    let mut modules = Vec::new();
    let mut id_map = IdMap::new();
    let mut errors = Errors::new();
    modules.push(module.check(&mut id_map, &mut errors));
    let main_id = get_main_id(&modules, id_map, &mut errors)?;
    if errors.present() {
        Err(Fail)
    } else {
        Ok(hir::Program { main_id, modules })
    }
}

impl ast::Module {
    fn check(self, id_map: &mut IdMap, errors: &mut Errors) -> hir::Module {
        let mut info = ModuleInfo {
            items: HashMap::new(),
            source: self.source,
        };
        let funs = self
            .funs
            .into_iter()
            .enumerate()
            .map(|(i, f)| f.check(i, id_map, &mut info, errors))
            .collect();
        hir::Module {
            path: info.source.path,
            items: info.items,
            funs,
        }
    }
}

impl ast::Fun {
    fn check(
        self,
        index: usize,
        id_map: &mut IdMap,
        module_info: &mut ModuleInfo,
        errors: &mut Errors,
    ) -> hir::Fun {
        let id = id_map.get(module_info.source.code[self.name].into());
        module_info.items.insert(id, index);
        let mut blocks = Vec::new();
        let mut typ_vars = Vec::new();
        let mut infer_checker = InferChecker::new();
        let hint = match self.ret_typ {
            Some(typ) => typ.check(errors),
            None => hir::Typ::Any,
        };
        let expr = self.body.check(hint, &mut typ_vars, &mut infer_checker);
        let ret_typ = expr.typ();
        blocks.push(hir::Block {
            out: hir::Out::Ret(expr),
        });
        infer_checker.run(&mut typ_vars, errors, module_info);
        hir::Fun {
            typ_vars,
            ret_typ,
            blocks,
        }
    }
}

impl ast::Typ {
    fn check(self, errors: &mut Errors) -> hir::Typ {
        match self {
            ast::Typ::Name(_) => hir::Typ::Err,
            ast::Typ::I32 => hir::IntTyp::I32.into(),
        }
    }
}

impl ast::Block {
    fn check(
        self,
        hint: hir::Typ,
        typ_vars: &mut Vec<hir::Typ>,
        infer_checker: &mut InferChecker,
    ) -> hir::Expr {
        self.out.check(hint, typ_vars, infer_checker)
    }
}

impl ast::Expr {
    fn check(
        self,
        hint: hir::Typ,
        typ_vars: &mut Vec<hir::Typ>,
        infer_checker: &mut InferChecker,
    ) -> hir::Expr {
        match self {
            ast::Expr::Int(int) => int.check(hint, typ_vars, infer_checker),
        }
    }
}

impl ast::Int {
    fn check(
        self,
        hint: hir::Typ,
        typ_vars: &mut Vec<hir::Typ>,
        infer_checker: &mut InferChecker,
    ) -> hir::Expr {
        let mut typ = if hint.is_int() { hint } else { hir::Typ::Any };
        if !typ.is_rigid() {
            typ_vars.push(typ);
            typ = hir::Typ::Var(typ_vars.len());
            infer_checker.checks.push(InferCheck {
                id: typ_vars.len(),
                loc: self.loc,
            });
        }
        hir::Expr::Int(typ, self.val)
    }
}

fn get_main_id(
    modules: &[hir::Module],
    mut id_map: IdMap,
    errors: &mut Errors,
) -> Result<hir::FunID, Fail> {
    let mod_id = 0;
    match modules[0].items.get(&id_map.get("main".into())) {
        Some(&method_id) => Ok(hir::FunID { mod_id, method_id }),
        None => Err(errors.no_main(&modules[0].path)),
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

struct InferChecker {
    checks: Vec<InferCheck>,
}

impl InferChecker {
    fn new() -> Self {
        Self { checks: Vec::new() }
    }

    fn run(self, typ_vars: &mut Vec<hir::Typ>, errors: &mut Errors, module_info: &ModuleInfo) {
        for check in self.checks {
            if !typ_vars[check.id].is_rigid() {
                errors.cannot_infer(&module_info.source, check.loc);
            }
        }
    }
}

struct InferCheck {
    id: usize,
    loc: Loc,
}
