#![allow(warnings, dead_code)]

mod emit;
mod parser;
mod visit;

use emit::EmitContextImpl;

use crate::emit::EmitDefault;
use crate::parser::esper_parser;

fn main() {
    let source = include_str!("../tests/emit.esp");

    match esper_parser::program(source) {
        Ok(program) => {
            // dbg!(&program);
            let mut ctx = EmitContextImpl::new();
            // ctx.use_prelude = true;
            let mut out = EmitDefault { ctx };
            let out = out.emit_program(&program, "test".into());
            println!("{}", out);
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}
