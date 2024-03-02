// static SHADER: &'static str = include_str!("whiteboard-0.js");

#![feature(let_chains)]
#![feature(try_blocks)]

use resast::prelude::*;
use ressa::*;
use std::{fs, io::Write};

use std::collections::HashMap;

use std::thread;

const STACK_SIZE: usize = 4 * 1024 * 1024;

pub mod class_bindings;
use class_bindings::{ClassBinding, ClassBindingObf};

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
    
    fs::write(
        "translations.json",
        serde_json::to_string(&translations).unwrap(),
    )
    .unwrap();

    // for a in &ast {
    //     comb_prog_part(a);
    // }

    let class_bindings: Vec<ClassBinding> = ast
        .iter()
        .flat_map(|a| class_bindings::visit_prog_part(a))
        .map(|binding| binding.convert(&translations))
        .collect();
    fs::write(
        "class_bindings.json",
        serde_json::to_string(&class_bindings).unwrap(),
    )
    .unwrap();

    {
        // Raw file
        let mut file = fs::File::options().write(true).create(true).open("classes.txt").unwrap();
        for binding in &class_bindings {
            file.write_all((binding.package.clone() + "." + &binding.class_name + "\n").as_bytes())
                .unwrap();
        }
    }

    println!("Done.");

    println!("Number of translations: {}", translations.len());
    println!("Number of class bindings: {} / 635", class_bindings.len());
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