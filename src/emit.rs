use crate::parser::Expr;
use crate::visit::{EsperContext, Visitor};

pub struct EmitContextImpl {
    pub output: String,
}

impl EmitContextImpl {
    pub fn new() -> Self {
        EmitContextImpl {
            output: String::new(),
        }
    }

    pub fn emit(&mut self, code: &str) {
        self.output.push_str(code);
        self.output.push('\n');
    }
}

impl EsperContext for EmitContextImpl {
    fn new() -> Self {
        EmitContextImpl::new()
    }
}

pub struct EmitDefault;

impl EmitDefault {
    pub fn emit_program(&self, expr: &Expr) -> String {
        let mut ctx = EmitContextImpl::new();
        self.emit_expr(&mut ctx, expr);
        ctx.output
    }

    pub fn emit_expr(&self, ctx: &mut EmitContextImpl, expr: &Expr) {
        match expr {
            Expr::Let(var, value) => {
                ctx.emit(&format!("auto {} = {};", var, self.emit_value(value)));
            }
            _ => {
                ctx.emit(&self.emit_value(expr));
            }
        }
    }

    fn emit_value(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(n) => n.to_string(),
            Expr::Float(f) => f.to_string(),
            Expr::Bool(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Expr::Char(c) => format!("'{}'", c),
            Expr::String(s) => format!("\"{}\"", s),
            Expr::Var(var_name) => var_name.clone(),
            _ => String::new(),
        }
    }
}
