#![allow(dead_code)]

mod parser;

use crate::parser::intrinsic_parser;

fn main() {
    let source = r#"
        let n = 2
        let p = 3
    "#;

    match intrinsic_parser::program(source) {
        Ok(program) => {
            println!("{:?}", program);
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}
