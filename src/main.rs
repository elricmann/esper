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
        r = (2 < (3))
        (k > 1)

        let s = (3 + 4)
        let t = (3 / (s * 2))

        let u = if a > 2 then 2 end
        let v = if b < 2 then 2 - 3 else 3 end

        if true then false else true end
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
