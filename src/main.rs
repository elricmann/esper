#![allow(dead_code)]

mod parser;

use crate::parser::intrinsic_parser;

fn main() {
    let source = r#"
        let add = |a, b|
          a + b
        end

        let k = |a, b|
          b = a
          c = a
        end
    "#;

    match intrinsic_parser::program(source) {
        Ok(program) => {
            dbg!(program);
        }

        Err(err) => {
            println!("{}", err);
        }
    }
}
