#![allow(dead_code)]

mod parser;
mod visit;

use crate::parser::esper_parser;

fn main() {
    let source = include_str!("../tests/parse.esp");

    match esper_parser::program(source) {
        Ok(program) => {
            dbg!(program);
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}
