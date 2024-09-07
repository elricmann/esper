#![allow(dead_code)]

mod parser;

use crate::parser::intrinsic_parser;

fn main() {
    let source = r#"
        let n = 2
        let p = 3
        n = p
        p = [2, 3, 4]
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
