use peg::parser;

parser! {
  pub grammar esper_parser() for str {
    rule typed_expr() -> Expr
      = typed_literal() / typed_symbol() / typed_variant()

    rule typed_literal() -> Expr
      = ty:(integer_literal() / float_literal() / bool_literal())
        { Expr::TypedLiteral(ty.into()) }

    rule typed_symbol() -> Expr
      = id:identifier() { Expr::TypedSymbol(id.into()) }

    rule type_alias() -> Expr
    = "type" _ id:identifier() _ "=" _ type_expr:typed_expr() _ "end" {
        Expr::TypeAlias(id.into(), vec![], Box::new(type_expr))
      }
    / "type" _ id:identifier() _ ty:type_generic() _ "=" _ type_expr:typed_expr() _ "end" {
        Expr::TypeAlias(id.into(), ty, Box::new(type_expr))
      }

    rule type_generic() -> Vec<Expr>
      = "<" _ args:(typed_expr() ** (_ "," _)) _ ">" {
      args
    }

    rule typed_variant() -> Expr
      = "|" _ first:typed_expr() _ "|"  _ rest:(typed_expr())* {
        let mut expr = first;

        for variant in rest {
            expr = Expr::TypedVariant(Box::new(expr), Box::new(variant));
        }

        expr
    }

    rule _() = (whitespace() / comment())*

    rule newline() = quiet!{ ['\n' | '\r']+ }

    rule whitespace() = quiet!{[' ' | '\t' | '\n' | '\r']+}

    rule comment() = "(*" (!"*)" [_])* "*)"

    rule identifier() -> &'input str
      = quiet!{s:$(['a'..='z' | 'A'..='Z' | '_']+)} / expected!("identifier")
      // { s.into() }

      rule float_literal() -> Expr
      = sign:("-")? n:$(['0'..='9']+ "." ['0'..='9']*) {
          let mut num = n.parse::<f64>().unwrap();

          if sign.is_some() {
              num = -num;
          }

          Expr::Float(num)
      }

    rule integer_literal() -> Expr
      = sign:("-")? n:$(['0'..='9']+) {
          let mut num = n.parse::<i64>().unwrap();

          if sign.is_some() {
              num = -num;
          }

          Expr::Int(num)
      }

    rule bool_literal() -> Expr
      = "true" { Expr::Bool(true) }
      / "false" { Expr::Bool(false) }

    rule string_literal() -> Expr
      = "\"" value:$([^ '"' ]*) "\"" {
      Expr::String(value.into())
    }

    rule char_literal() -> Expr
      = "'" value:$([^ '\'' ]) "'" {
      Expr::Char(value.chars().next().unwrap())
    }

    rule directive_expr() -> Expr
      = "@" _ directive:(call_expr() / identifier_expr()) _ expr:primary() {
      Expr::Directive(Box::new(directive), Box::new(expr))
    }

    rule range_expr() -> Expr
      = start:(integer_literal() / identifier_expr())
        ".."
        end:(integer_literal() / identifier_expr()) {
          Expr::Range(Box::new(start), Box::new(end))
      }

    rule member_expr() -> Expr
    =
    // base:identifier_expr() (_ "." _) rest:identifier_expr() {
    //     let mut members = vec![base, rest];
    //     Expr::Member(members)
    //   }
    // /
    base:identifier_expr() (_ "." _) rest:(member_expr())* {
        let mut members = vec![base];
        members.extend(rest);
        Expr::Member(members)
    }

    rule call_expr() -> Expr
    = callee:(fn_expr() / identifier_expr() / member_expr()) "(" _? args:(expr() ** (_ "," _)) _? ")" {
        Expr::Call(Box::new(callee), args)
      }
    / callee:(identifier_expr() / member_expr()) _ ty:type_generic() _ "(" _? args:(expr() ** (_ "," _)) _? ")" {
      Expr::TypedCall(Box::new(callee), ty, args)
    }

    rule let_binding() -> Expr
      = "let" _ id:identifier() _ "=" _ expr:expr() { Expr::Let(id.into(), Box::new(expr)) }
      / "let" _ id:identifier() _ ":" _ ty:typed_expr() _ "=" _ expr:expr() {
        Expr::TypedLet(id.into(), Box::new(ty), Box::new(expr))
      }

    rule assignable() -> Expr
      = id:identifier_expr() { id }

    rule assign() -> Expr
      = lhs:assignable() _ "=" _ rhs:expr() { Expr::Assign(Box::new(lhs), Box::new(rhs)) }

    // assign must hold the highest precedence
    rule primary() -> Expr
      = assign() / paren_expr() / directive_expr() / struct_expr() / type_alias() / call_expr() /
        range_expr() / member_expr() /
        loop_expr() / if_expr() / fn_expr() / let_binding() / bool_literal() /
        float_literal() / integer_literal() / string_literal() / char_literal() /
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

    rule struct_expr() -> Expr
      = "struct" _ id:identifier() _ entries:(struct_entry() ** (_ "," _)) _ "end" {
      let entries = entries.into_iter().collect();
      Expr::Struct(id.into(), entries)
    }

    rule struct_entry() -> (String, Expr)
    = prop:identifier() _ ":" _ type_:typed_expr() {
      (prop.into(), type_)
    }
    / method:identifier() _ ":" _ fn_:fn_expr() {
      (method.into(), fn_)
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
      = "if" _ cond:expr() _ "then" _ then_body:body_expr() _ "end" {
      Expr::If(Box::new(cond), then_body, None)
    }
      / "if" _ cond:expr() _ "then" _ then_body:body_expr() _ "else" _ else_body:body_expr() _ "end" {
      Expr::If(Box::new(cond), then_body, Some(else_body))
    }

    rule loop_expr() -> Expr
      = "loop" _ loop_var:expr() _ "in" _ iter:primary() _ body:body_expr() _ "end" {
        Expr::Loop(Box::new(loop_var), Box::new(iter), body)
    }

    rule fn_expr() -> Expr
      = "|" _ args:(identifier() ** (_ "," _)) _ "|" _ body:body_expr() _ "end" {
      Expr::Fn(args.into_iter().map(|arg| arg.into()).collect(), body)
    }

    rule body_expr() -> Vec<Expr>
      = expr() ** (_ ";" _)

    rule exprs_list() -> Vec<Expr>
      = first:(expr() / primary()) newline()? rest:(expr() / primary())* {
        let mut exprs = vec![first];
        exprs.extend(rest);
        exprs
    }

    pub rule program() -> Expr
      = _ exprs:(expr() ** _) _ {
        let boxed_exprs = exprs
          .into_iter()
          .map(|e| Box::new(e) as Box<Expr>)
          .collect::<Vec<Box<Expr>>>();
        Expr::Program(boxed_exprs)
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Program(Vec<Box<Expr>>),
    Let(String, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Var(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    // @todo: box members
    List(Vec<Expr>),
    Record(Vec<Vec<Expr>>),
    Range(Box<Expr>, Box<Expr>),
    Directive(Box<Expr>, Box<Expr>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    // @fix: precedence of >= <=
    Compare(Box<Expr>, CompareOp, Box<Expr>),
    If(Box<Expr>, Vec<Expr>, Option<Vec<Expr>>),
    Loop(Box<Expr>, Box<Expr>, Vec<Expr>),
    Fn(Vec<String>, Vec<Expr>),
    Member(Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Struct(String, Vec<(String, Expr)>),
    TypedSymbol(String),
    TypedLiteral(Box<Expr>),
    TypedVariant(Box<Expr>, Box<Expr>),
    TypedLet(String, Box<Expr>, Box<Expr>),
    TypeAlias(String, Vec<Expr>, Box<Expr>),
    TypedCall(Box<Expr>, Vec<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
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
