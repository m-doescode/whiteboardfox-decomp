// static SHADER: &'static str = include_str!("whiteboard-0.js");

#![feature(let_chains)]

use resast::prelude::*;
use ressa::*;
use std::fs;

use std::collections::HashMap;

use std::thread;

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

    // dbg!(&ast[0..10]);
    // dbg!(&ast[0]);

    let mut translations: HashMap<String, String> = HashMap::new();

    for a in ast {
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
                if let Pat::Ident(Ident { name: identifier }) = decl.id
                    && let Some(Expr::Lit(Lit::String(StringLit::Single(mapping)))) = decl.init
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
