use crate::parser::{BinOp, CompareOp, Expr};
use crate::visit::{EsperContext, Visitor};

pub struct EmitContextImpl {
    pub level: usize,
    pub output: String,
    pub module: String,
}

impl EmitContextImpl {
    pub fn new() -> Self {
        EmitContextImpl {
            level: 0,
            output: String::new(),
            module: String::new(),
        }
    }

    pub fn emit(&mut self, code: &str) {
        self.output.push_str(code);
        self.output.push('\n');
    }

    pub fn indent(&mut self) -> String {
        " ".repeat(self.level)
    }
}

impl EsperContext for EmitContextImpl {
    fn new() -> Self {
        EmitContextImpl::new()
    }
}

pub struct EmitDefault;

impl EmitDefault {
    pub fn emit_program(&self, expr: &Expr, module_id: &str) -> String {
        let mut ctx = EmitContextImpl::new();
        ctx.module = module_id.into();
        self.emit_expr(&mut ctx, expr);
        ctx.output
    }

    pub fn emit_expr(&self, ctx: &mut EmitContextImpl, expr: &Expr) {
        match expr {
            Expr::Program(exprs) => {
                ctx.emit(&format!("namespace {} {{", ctx.module));
                // ctx.level = 2;
                // ctx.emit("");

                for sub_expr in exprs {
                    self.emit_expr(ctx, sub_expr);
                }

                // ctx.emit("");
                ctx.emit(&format!("}} // namespace {}", ctx.module));
            }

            Expr::Let(var, value) => {
                let indent = ctx.indent();

                match value.as_ref() {
                    Expr::Fn(params, body) => {
                        ctx.emit("");
                        let params_str = params
                            .iter()
                            .map(|(param, ty)| match ty {
                                Some(ty) => format!("{} {}", self.emit_type(ty), param),
                                None => param.clone(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        ctx.emit(&format!("{}auto {}({}) {{", indent, var, params_str));
                        ctx.level += 2;

                        let last = body.last(); /* keep before redefinition */
                        let body: &Vec<Expr> = &body[0..body.len() - 1].into();

                        for expr in body {
                            self.emit_expr(ctx, expr);
                        }

                        if let Some(last) = last {
                            let indent = ctx.indent();
                            ctx.emit(&format!("{}return {};", indent, self.emit_value(last)));
                        }

                        ctx.level -= 2;
                        ctx.emit(&format!("{}}}", indent));
                    }
                    _ => {
                        ctx.emit(&format!(
                            "{}auto {} = {};",
                            indent,
                            var,
                            self.emit_value(value)
                        ));
                    }
                }
            }

            Expr::TypedLet(var, ty, value) => {
                let indent = ctx.indent();

                match value.as_ref() {
                    Expr::Fn(params, body) => {
                        ctx.emit("");
                        let params_str = params
                            .iter()
                            .map(|(param, ty)| match ty {
                                Some(ty) => format!("{} {}", self.emit_type(ty), param),
                                None => param.clone(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        let return_type = self.emit_type(ty);

                        ctx.emit(&format!(
                            "{}{} {}({}) {{",
                            indent, return_type, var, params_str
                        ));
                        ctx.level += 2;

                        let last = body.last();
                        let body: &Vec<Expr> = &body[0..body.len() - 1].into();

                        for expr in body {
                            self.emit_expr(ctx, expr);
                        }

                        if let Some(last) = last {
                            let indent = ctx.indent();
                            ctx.emit(&format!("{}return {};", indent, self.emit_value(last)));
                        }

                        ctx.level -= 2;
                        ctx.emit(&format!("{}}}", indent));
                    }
                    _ => {
                        let ty_str = self.emit_type(ty);
                        ctx.emit(&format!(
                            "{}{} {} = {};",
                            indent,
                            ty_str,
                            var,
                            self.emit_value(value)
                        ));
                    }
                }
            }

            Expr::Assign(lhs, rhs) => {
                let indent = ctx.indent();

                ctx.emit(&format!(
                    "{}{} = {};",
                    indent,
                    self.emit_value(lhs),
                    self.emit_value(rhs)
                ));
            }

            Expr::Call(callee, args) => {
                let indent = ctx.indent();
                let callee_str = self.emit_value(callee);
                let args_str = args
                    .iter()
                    .map(|arg| self.emit_value(arg))
                    .collect::<Vec<_>>()
                    .join(", ");

                ctx.emit(&format!("{}{}({});", indent, callee_str, args_str));
            }

            _ => {
                let indent = ctx.indent();
                ctx.emit(&format!("{}{};", indent, &self.emit_value(expr)));
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

            Expr::Bin(lhs, op, rhs) => {
                let lhs_str = self.emit_value(lhs);
                let rhs_str = self.emit_value(rhs);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                };

                format!("({} {} {})", lhs_str, op_str, rhs_str)
            }

            Expr::Compare(lhs, op, rhs) => {
                let lhs_str = self.emit_value(lhs);
                let rhs_str = self.emit_value(rhs);
                let op_str = match op {
                    CompareOp::Gt => ">",
                    CompareOp::Lt => "<",
                    CompareOp::Gte => ">=",
                    CompareOp::Lte => "<=",
                };

                format!("({} {} {})", lhs_str, op_str, rhs_str)
            }

            _ => String::new(),
        }
    }

    fn emit_type(&self, ty: &Expr) -> String {
        match ty {
            Expr::TypedSymbol(type_name) => type_name.clone(),
            Expr::TypedLiteral(type_name) => {
                format!("decltype({})", self.emit_value(type_name))
            }
            _ => String::new(),
        }
    }
}