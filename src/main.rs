#![allow(dead_code)]

mod parser;

use crate::parser::intrinsic_parser;

fn main() {
    let source = r#"
        let n = 2
        let p = 3
        n = p
        p = [2, 3, 4]
        let k = { p: 3, q: 4 }
        let q = { 2: p, 4: q }
        r = (2 < 3)

        (k > 1)
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
