use peg::parser;

parser! {
  pub grammar intrinsic_parser() for str {
    rule _() = whitespace()?

    rule newline() = quiet!{ ['\n' | '\r']+ }

    rule whitespace() = quiet!{[' ' | '\t' | '\n' | '\r']+}

    rule identifier() -> &'input str
      = quiet!{s:$(['a'..='z' | 'A'..='Z' | '_']+)} / expected!("identifier")
      // { s.into() }

    rule integer_literal() -> Expr
      = n:$(['0'..='9']+) { Expr::Int(n.parse().unwrap()) }

    rule bool_literal() -> Expr
      = "true" { Expr::Bool(true) }
      / "false" { Expr::Bool(false) }

    rule range_expr() -> Expr
      = start:(integer_literal() / identifier_expr()) ".." end:(integer_literal() / identifier_expr()) {
          Expr::Range(Box::new(start), Box::new(end))
      }

    rule let_binding() -> Expr
      = "let" _ id:identifier() _ "=" _ expr:expr() { Expr::Let(id.into(), Box::new(expr)) }

    rule assignable() -> Expr
      = id:identifier_expr() { id }

    rule assign() -> Expr
      = lhs:assignable() _ "=" _ rhs:expr() { Expr::Assign(Box::new(lhs), Box::new(rhs)) }

    // assign must hold the highest precedence
    rule primary() -> Expr
      = assign() / paren_expr() / range_expr() / if_expr() / fn_expr() / let_binding() / bool_literal() / integer_literal() /
        identifier_expr() / list() / record()

    rule expr() -> Expr
      = add_sub() / compare() / primary()

    rule paren_expr() -> Expr
      = "(" _ e:expr() _ ")" { e }

    rule identifier_expr() -> Expr
      = id:identifier() { Expr::Var(id.into()) }

    rule list() -> Expr
      = "[" lst:(expr() ** (_ "," _)) "]" { Expr::List(lst) }

    rule record_key() -> Expr
      = identifier_expr() / integer_literal()

    rule record_entry() -> (Expr, Expr)
      = key:record_key() _ ":" _ value:expr() { (key, value) }

    rule record() -> Expr
      = "{" _ entries:(record_entry() ** (_ "," _)) _ "}" {
      let kv_pairs = entries.into_iter().map(|(key, value)| vec![key, value]).collect();

      Expr::Record(kv_pairs)
    }

    rule add_sub() -> Expr
      = lhs:mul_div() _ op:$("+" / "-") _ rhs:mul_div() {
        let op_enum = match op {
          "+" => BinOp::Add,
          "-" => BinOp::Sub,
          _ => unreachable!(),
        };

        Expr::Bin(Box::new(lhs), op_enum, Box::new(rhs))
    } / mul_div()

    rule mul_div() -> Expr
      = lhs:primary() _ op:$("*" / "/") _ rhs:primary() {
        let op_enum = match op {
          "*" => BinOp::Mul,
          "/" => BinOp::Div,
          _ => unreachable!(),
        };

        Expr::Bin(Box::new(lhs), op_enum, Box::new(rhs))
    } / compare()

    rule compare_op() -> &'input str
      = op:$(">" / "<" / ">=" / "<=") { op }

    rule compare() -> Expr
      = lhs:primary() _ op:compare_op() _ rhs:primary() {
        let op_enum = match op {
          ">" => CompareOp::Gt,
          "<" => CompareOp::Lt,
          ">=" => CompareOp::Gte,
          "<=" => CompareOp::Lte,
          _ => unreachable!(),
        };

        Expr::Compare(Box::new(lhs), op_enum, Box::new(rhs))
    } / primary()

    rule if_expr() -> Expr
      = "if" _ cond:expr() _ "then" _ then_body:expr() _ "end" {
      Expr::If(Box::new(cond), Box::new(then_body), None)
    }
      / "if" _ cond:expr() _ "then" _ then_body:expr() _ "else" _ else_body:expr() _ "end" {
      Expr::If(Box::new(cond), Box::new(then_body), Some(Box::new(else_body)))
    }

    rule fn_expr() -> Expr
      = "|" _ args:(identifier() ** (_ "," _)) _ "|" _ body:exprs_list() _ "end" {
      Expr::Fn(args.into_iter().map(|arg| arg.into()).collect(), body)
    }

    // rule expr_inner() -> Expr
    //   = _ first:expr() _ "in" { first }

    rule exprs_list() -> Vec<Expr>
      = first:(expr() / primary()) newline()? rest:(expr() / primary())* {
        let mut exprs = vec![first];
        exprs.extend(rest);
        exprs
    }

    pub rule program() -> Vec<Expr>
      = _ exprs:(expr() ** _) _ { exprs }
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Let(String, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Var(String),
    Int(i64),
    Bool(bool),
    // @todo: box members
    List(Vec<Expr>),
    Record(Vec<Vec<Expr>>),
    Range(Box<Expr>, Box<Expr>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    // @fix: precedence of >= <=
    Compare(Box<Expr>, CompareOp, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Fn(Vec<String>, Vec<Expr>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CompareOp {
    Gt,
    Lt,
    Gte,
    Lte,
}
