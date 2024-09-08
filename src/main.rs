#![allow(dead_code)]

mod parser;

use crate::parser::intrinsic_parser;

fn main() {
    let source = r#"
        let add = |a, b| a + b end

        let m = |a, b|
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

        a(2, 3, n + 1)
        let p = q(r..s, t..u)

        w(x.y)

        3.14159
        (-2.9)
        7
        (-8)

        let v : 0 = 0
        let n : int = 1
        let p : | bool | float = 0.0

        type a = b end
        type c =
          | d
          | e
        end

        type f<g, h> = i end
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
