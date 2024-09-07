use peg::parser;

parser! {
  pub grammar intrinsic_parser() for str {
    rule whitespace() = quiet!{[' ' | '\t' | '\n' | '\r']+}

    rule identifier() -> &'input str
      = quiet!{s:$(['a'..='z' | 'A'..='Z' | '_']+)} / expected!("identifier")
      // { s.into() }

    rule integer_literal() -> Expr
      = n:$(['0'..='9']+) { Expr::Int(n.parse().unwrap()) }

    rule let_binding() -> Expr
      = "let" _ id:identifier() _ "=" _ expr:expr() { Expr::Let(id.into(), Box::new(expr)) }

    rule assignable() -> Expr
      = id:identifier_expr() { id }

    rule assign() -> Expr
      = lhs:assignable() _ "=" _ rhs:expr() { Expr::Assign(Box::new(lhs), Box::new(rhs)) }

    // assign must hold the highest precedence
    rule primary() -> Expr
      = assign() / let_binding() / integer_literal() / identifier_expr() / list() / record()
    
    rule expr() -> Expr
      = compare() / primary()

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

    rule compare_op() -> &'input str
    = op:$(">" / "<" / ">=" / "<=") { op }

    rule compare() -> Expr
    = lhs:primary() _ op:compare_op() _ rhs:primary() {
      match op {
        ">" => Expr::Gt(Box::new(lhs), Box::new(rhs)),
        "<" => Expr::Lt(Box::new(lhs), Box::new(rhs)),
        ">=" => Expr::Gte(Box::new(lhs), Box::new(rhs)),
        "<=" => Expr::Lte(Box::new(lhs), Box::new(rhs)),
        _ => unreachable!(),
      }
    }

    rule _() = whitespace()?

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
    // @todo: box members
    List(Vec<Expr>),
    Record(Vec<Vec<Expr>>),
    Gt(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
}
