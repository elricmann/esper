#![allow(dead_code)]

mod parser;

use crate::parser::intrinsic_parser;

fn main() {
    let source = r#"
        let add = |a, b| a + b end

        let k = |a, b|
          b = a
        end

        let t = true
        let f = false

        let r = 2..3
        let s = i..n

        if a > 2 then true else false end

        loop i in 2..3
          i * 2
        end

        let a = b.c
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
