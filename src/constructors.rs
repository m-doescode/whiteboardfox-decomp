use resast::prelude::*;
use ressa::*;

use serde::{Deserialize, Serialize};
use std::iter;

use std::collections::HashMap;

use crate::class_bindings::ClassBinding;

pub fn visit_prog_part(a: &ProgramPart) -> Vec<ConstructorBindingObf> {
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

fn visit_stmt(stmt: &Stmt) -> Vec<ConstructorBindingObf> {
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

fn visit_expr(expr: &Expr) -> Vec<ConstructorBindingObf> {
    if let Expr::Call(call) = expr
        && let Expr::Ident(func_name) = *call.callee.clone()
        && func_name.name == "SB" /* Hard-coded, for now. */

        && call.arguments.len() == 4 // Some classes are not bound to a constructor.
        && let Expr::Lit(Lit::Number(class_id_unparsed)) = &call.arguments[0]
        && let Expr::Ident(Ident { name: constructor }) = &call.arguments[3]
    {
        vec![ConstructorBindingObf {
            class_id: class_id_unparsed.to_string().parse::<u16>().unwrap(),
            constructor_name: constructor.to_string(),
        }]
    } else {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConstructorBindingObf {
    class_id: u16,
    constructor_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConstructorBinding {
    pub class_id: u16,
    pub constructor_name: String,
    pub class_qualifier: String,
}

impl ConstructorBindingObf {
    pub fn convert(self, translations: &HashMap<u16, &ClassBinding>) -> ConstructorBinding {
        if let Some(class_binding) = translations.get(&self.class_id) {
            ConstructorBinding {
                class_id: self.class_id,
                constructor_name: self.constructor_name,
                class_qualifier: class_binding.package.to_string()
                    + "."
                    + &class_binding.class_name,
            }
        } else {
            // No debug information provided for constructor
            ConstructorBinding {
                class_id: self.class_id,
                constructor_name: self.constructor_name,
                class_qualifier: "?".to_string(),
            }
        }
    }
}
