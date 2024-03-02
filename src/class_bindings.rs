
use resast::prelude::*;
use ressa::*;

use serde::{Deserialize, Serialize};
use std::iter;

use std::collections::HashMap;

pub fn visit_prog_part(a: &ProgramPart) -> Vec<ClassBindingObf> {
    match a {
        ProgramPart::Decl(Decl::Var(_, decls)) => decls
            .iter()
            .flat_map(|decl| decl.init.as_ref().map_or(vec![], |expr| visit_expr(expr)))
            .collect(),
        // 10 missing instantiations
        ProgramPart::Stmt(stmt) => visit_stmt(stmt),
        _ => vec![],
    }
}

fn visit_stmt(stmt: &Stmt) -> Vec<ClassBindingObf> {
    match stmt {
        Stmt::Expr(expr) => visit_expr(expr),
        // Stmt::Throw(expr) => visit_expr(expr),

        // Stmt::Block(BlockStmt(stmts)) => stmts.iter().flat_map(|a| visit_prog_part(a)).collect(),
        // Stmt::With(WithStmt { object: _, body }) => visit_stmt(body),
        // Stmt::Labeled(LabeledStmt { label: _, body }) => visit_stmt(body),
        // Stmt::If(IfStmt { test: _, consequent, alternate }) => { let mut a = visit_stmt(consequent); if let Some(b) = alternate { a.extend(visit_stmt(b)); } a },
        // Stmt::Return(expr) => expr.as_ref().map_or(vec![], |expr| visit_expr(expr)),
        // Stmt::Switch(SwitchStmt { discriminant: _, cases }) => cases.iter().flat_map(|case| case.consequent.iter().flat_map(|a| visit_prog_part(a)).chain(case.test.as_ref().map_or(vec![], |expr| visit_expr(expr)))).collect(),
        // Stmt::Try(TryStmt { block: BlockStmt(stmts), handler, finalizer }) => stmts.iter().flat_map(|a| visit_prog_part(a)).collect(),

        // TODO: Many more unhandled here but I couldn't give less of a fuck. This was a *pain*
        _ => vec![],
    }
}

fn visit_expr(expr: &Expr) -> Vec<ClassBindingObf> {
    if let Expr::Call(call) = expr
        && let Expr::Ident(func_name) = *call.callee.clone()
        && func_name.name == "M5" /* Hard-coded, for now. */

        && let Expr::Ident(package_obf) = &call.arguments[0]
        && let Expr::Lit(Lit::String(StringLit::Single(class_name))) = &call.arguments[1]
        && let Expr::Lit(Lit::Number(obj_id_unparsed)) = &call.arguments[2]
    {
        vec![ClassBindingObf {
            package_obf: package_obf.name.to_string(),
            class_name: class_name.to_string(),
            class_id: obj_id_unparsed.to_string().parse::<u16>().unwrap(),
        }]
    } else {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClassBindingObf {
    package_obf: String,
    class_name: String,
    class_id: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClassBinding {
    pub package_obf: String,
    pub package: String,
    pub class_name: String,
    pub class_id: u16,
}

impl ClassBindingObf {
    pub fn convert(self, translations: &HashMap<String, String>) -> ClassBinding {
        ClassBinding {
            package: translations.get(&self.package_obf).unwrap().to_string(),
            package_obf: self.package_obf,
            class_name: self.class_name,
            class_id: self.class_id,
        }
    }
}