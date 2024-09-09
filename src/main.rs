#![allow(warnings, dead_code)]

mod emit;
mod parser;
mod visit;

use crate::emit::EmitDefault;
use crate::parser::esper_parser;

fn main() {
    let source = include_str!("../tests/emit.esp");

    match esper_parser::program(source) {
        Ok(program) => {
            dbg!(&program);

            let out = EmitDefault {};
            let out = out.emit_program(&program);
            dbg!(out);
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}
