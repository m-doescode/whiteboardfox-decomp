// static SHADER: &'static str = include_str!("whiteboard-0.js");

#![feature(let_chains)]
#![feature(try_blocks)]

use resast::prelude::*;
use ressa::*;
use std::fs;

use std::collections::HashMap;

use std::thread;

use serde::{ Serialize, Deserialize };
use std::iter;

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn run() {
    // let input = fs::read_to_string("test.js").unwrap();
    let input = fs::read_to_string("whiteboard-0.js").unwrap();
    let mut parser = Parser::new(&input).unwrap();
    let pgm = parser.parse().unwrap();
    let ast = match pgm {
        Program::Mod(_) => panic!(),
        Program::Script(parts) => parts,
    };

    println!("Analyzing...");

    // dbg!(&ast[0..10]);
    // dbg!(&ast[0]);

    let mut translations: HashMap<String, String> = HashMap::new();

    for a in &ast {
        // match a {
        //     ProgramPart::Decl(d) => match d {
        //         Decl::Var(kind, )
        //         _ => {},
        //     },
        //     _ => {},
        // }
        if let ProgramPart::Decl(Decl::Var(kind, decls)) = a {
            // if decls.len() > 6 {
            for decl in decls {
                // dbg!(decl);
                if let Pat::Ident(Ident { name: identifier }) = &decl.id
                    && let Some(Expr::Lit(Lit::String(StringLit::Single(mapping)))) = &decl.init
                {
                    // print!("this many");
                    translations.insert(identifier.to_string(), mapping.to_string());
                }
            }
            // }
        }
    }

    let packages: HashMap<&String, &String> = translations
        .iter()
        .filter(|&(_, package)| package.starts_with("java.") || package.starts_with("com."))
        .collect();

    fs::write(
        "translations.json",
        serde_json::to_string(&translations).unwrap(),
    )
    .unwrap();
    fs::write("packages.json", serde_json::to_string(&packages).unwrap()).unwrap();

    // for a in &ast {
    //     comb_prog_part(a);
    // }

    let instantiations: Vec<Instantiation> = ast.iter().flat_map(|a| visit_prog_part(a)).map(|i11n| Instantiation {
        package: translations.get(&i11n.package_obf).unwrap().to_string(),
        package_obf: i11n.package_obf,
        class_name: i11n.class_name,
        object_id: i11n.object_id,
    }).collect();
    fs::write("instantiations.json", serde_json::to_string(&instantiations).unwrap()).unwrap();

    println!("Done.");

    println!("Number of translations: {}", translations.len());
    println!("Number of object instantiations: {}", instantiations.len());
}

fn main() {
    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

fn visit_prog_part(a: &ProgramPart) -> Vec<I11nObf> {
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

fn visit_stmt(stmt: &Stmt) -> Vec<I11nObf> {
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

fn visit_expr(expr: &Expr) -> Vec<I11nObf> {
    if let Expr::Call(call) = expr
        && let Expr::Ident(func_name) = *call.callee.clone()
        && func_name.name == "M5" /* Hard-coded, for now. */

        && let Expr::Ident(package_obf) = &call.arguments[0]
        && let Expr::Lit(Lit::String(StringLit::Single(class_name))) = &call.arguments[1]
        && let Expr::Lit(Lit::Number(obj_id_unparsed)) = &call.arguments[2]
    {
        vec![I11nObf {
            package_obf: package_obf.name.to_string(),
            class_name: class_name.to_string(),
            object_id: obj_id_unparsed.to_string().parse::<u16>().unwrap(),
        }]
    } else {
        vec![]
    }
}

#[derive(Serialize, Deserialize)]
struct I11nObf {
    package_obf: String,
    class_name: String,
    object_id: u16,
}

#[derive(Serialize, Deserialize)]
struct Instantiation {
    package_obf: String,
    package: String,
    class_name: String,
    object_id: u16,
}