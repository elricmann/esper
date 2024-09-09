#![allow(warnings, dead_code)]

mod emit;
mod parser;
mod visit;

use crate::parser::esper_parser;

fn main() {
    let source = include_str!("../tests/emit.esp");

    match esper_parser::program(source) {
        Ok(program) => {
            dbg!(program);
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}
