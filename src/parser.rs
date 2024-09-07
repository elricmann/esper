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

    // assign must hold the highest precedence
    rule expr() -> Expr
      = assign() / let_binding() / integer_literal() / identifier_expr() / list()

    rule identifier_expr() -> Expr
      = id:identifier() { Expr::Var(id.into()) }

    rule list() -> Expr
      = "[" lst:(expr() ** (_ "," _)) "]" { Expr::List(lst) }

    rule assignable() -> Expr
      = id:identifier_expr() { id }

    rule assign() -> Expr
      = lhs:assignable() _ "=" _ rhs:expr() { Expr::Assign(Box::new(lhs), Box::new(rhs)) }

    rule _() = whitespace()?

    pub rule program() -> Vec<Expr>
      = _ exprs:(expr() ** _) _ { exprs }
  }
}

#[derive(Debug)]
pub enum Expr {
    Let(String, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Var(String),
    Int(i64),
    // @todo: box members
    List(Vec<Expr>),
}
