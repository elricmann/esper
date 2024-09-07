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

    rule expr() -> Expr
      = let_binding() / integer_literal() / identifier_expr()

    rule identifier_expr() -> Expr
      = id:identifier() { Expr::Var(id.into()) }

    rule _() = whitespace()?

    pub rule program() -> Vec<Expr>
      = _ exprs:(expr() ** _) _ { exprs }
  }
}

#[derive(Debug)]
pub enum Expr {
    Let(String, Box<Expr>),
    Var(String),
    Int(i64),
}
