use crate::parser::{BinOp, CompareOp, Expr};
use crate::visit::{EsperContext, Visitor};

// note: esper outputs with some non-practical patterns:
// 1 - where GLIBXX is not defined or not in /usr/include/c++, we conditionally
//     include libstdc++ headers since we will only be compiling with clang++
// 2 - using namespace std is forced since the :: operator is reserved (lst slice)
// 3 - public class member definitions (leaky abstractions) is forced
// 4 - C++ initializer list for RHS list-like expressions

// @todo:
// 1 - type decls/alias for various
// 2 - member expr with trailing/leading callable
// 3 - constructors & destructors, copy constructor & move semantics
// 4 - refs & derefs for raw values and directives for smart ptrs
// 5 - operator overloading (maybe?)

#[derive(Debug, Clone)]
pub struct EmitContextImpl {
    pub level: usize,
    pub output: String,
    pub module: String,
    pub use_glibcxx: bool,
}

impl EmitContextImpl {
    pub fn new() -> Self {
        EmitContextImpl {
            level: 0,
            output: String::new(),
            module: String::new(),
            use_glibcxx: false,
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

#[derive(Debug, Clone)]
pub struct EmitDefault {
    pub ctx: EmitContextImpl,
}

impl EmitDefault {
    pub fn emit_program(&mut self, expr: &Expr, module_id: &str) -> String {
        let mut ctx = self.ctx.clone();

        ctx.module = module_id.into();
        self.emit_expr(&mut ctx, expr);
        ctx.output
    }

    pub fn emit_expr(&self, ctx: &mut EmitContextImpl, expr: &Expr) {
        match expr {
            Expr::Program(exprs) => {
                if ctx.use_glibcxx {
                    ctx.emit(include_str!("../lib/stdc++.h"));
                }

                ctx.emit("using namespace std;\n");
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
                            if matches!(
                                last,
                                Expr::Int(_)
                                    | Expr::Float(_)
                                    | Expr::Bool(_)
                                    | Expr::Char(_)
                                    | Expr::String(_)
                                    | Expr::Var(_)
                                    | Expr::Bin(_, _, _)
                                    | Expr::Compare(_, _, _)
                                    | Expr::List(_)
                                    | Expr::Member(_)
                                    | Expr::Range(_, _)
                                    | Expr::Call(_, _)
                                    | Expr::TypedCall(_, _, _)
                            ) {
                                let indent = ctx.indent();
                                ctx.emit(&format!("{}return {};", indent, self.emit_value(last)));
                            }
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
                            if matches!(
                                last,
                                Expr::Int(_)
                                    | Expr::Float(_)
                                    | Expr::Bool(_)
                                    | Expr::Char(_)
                                    | Expr::String(_)
                                    | Expr::Var(_)
                                    | Expr::Bin(_, _, _)
                                    | Expr::Compare(_, _, _)
                                    | Expr::List(_)
                                    | Expr::Member(_)
                                    | Expr::Range(_, _)
                                    | Expr::Call(_, _)
                                    | Expr::TypedCall(_, _, _)
                            ) {
                                let indent = ctx.indent();
                                ctx.emit(&format!("{}return {};", indent, self.emit_value(last)));
                            }
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

            Expr::If(cond, then_body, else_body) => {
                let cond_str = self.emit_value(cond);
                let indent = ctx.indent();

                ctx.emit(&format!("{}if ({}) {{", indent, cond_str));
                ctx.level += 2;

                for expr in then_body {
                    self.emit_expr(ctx, expr);
                }

                ctx.level -= 2;
                ctx.emit(&format!("{}}}", indent));

                if let Some(else_body) = else_body {
                    ctx.emit(&format!("{}else {{", indent));
                    ctx.level += 2;

                    for expr in else_body {
                        self.emit_expr(ctx, expr);
                    }

                    ctx.level -= 2;
                    ctx.emit(&format!("{}}}", indent));
                }
            }

            Expr::Loop(loop_var, iter_expr, body) => {
                let mut loop_var_str = self.emit_value(loop_var);
                let iter_str = self.emit_value(iter_expr);
                let indent = ctx.indent();

                if (matches!(*loop_var.to_owned(), Expr::List(_))) {
                    loop_var_str.replace_range(0..1, "[");
                    loop_var_str.replace_range(loop_var_str.len() - 1..loop_var_str.len(), "]");
                }

                ctx.emit(&format!(
                    "\n{}for (auto {} : {}) {{",
                    indent, loop_var_str, iter_str
                ));

                ctx.level += 2;

                for expr in body {
                    self.emit_expr(ctx, expr);
                }

                ctx.level -= 2;
                ctx.emit(&format!("{}}}", indent));
            }

            Expr::Struct(name, entries) => {
                let indent = ctx.indent();
                ctx.emit(&format!("\nclass {} {{", name));
                ctx.emit("public:");
                ctx.level += 2;

                for (field_name, expr) in entries {
                    match expr {
                        Expr::TypedSymbol(_) | Expr::TypedLiteral(_) => {
                            let indent = ctx.indent();
                            let field_ty = self.emit_type(expr);

                            ctx.emit(&format!("{}{} {};", indent, field_ty, field_name));
                        }

                        Expr::Fn(params, body) => {
                            let indent = ctx.indent();
                            let params_str = params
                                .iter()
                                .map(|(param, ty)| match ty {
                                    Some(ty) => format!("{} {}", self.emit_type(ty), param),
                                    None => param.clone(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ");

                            // @todo: method return type

                            ctx.emit(&format!("{}auto {}({}) {{", indent, field_name, params_str));
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
                        _ => {}
                    }
                }

                ctx.level -= 2;
                ctx.emit(&format!("{}}};", indent));
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

            Expr::Member(exprs) => self.emit_member(exprs),

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

            Expr::Range(lhs, rhs) => {
                let lhs_str = self.emit_value(lhs);
                let rhs_str = self.emit_value(rhs);

                format!("ranges::views::iota({},{})", lhs_str, rhs_str)
            }

            // RHS can assume the LHS casts the C++ initializer lists
            // or ideally use vector<T>() as an extended definition
            Expr::List(exprs) => {
                let elements = exprs
                    .iter()
                    .map(|e| self.emit_value(e))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{{}}}", elements)
            }

            Expr::Call(callee, args) => {
                let callee_str = self.emit_value(callee);
                let args_str = args
                    .iter()
                    .map(|arg| self.emit_value(arg))
                    .collect::<Vec<_>>()
                    .join(", ");

                return (format!("{}({})", callee_str, args_str));
            }

            Expr::TypedCall(callee, generics, args) => {
                let callee_str = self.emit_value(callee);

                let generics_str = if !generics.is_empty() {
                    let generics_ty_str = generics
                        .iter()
                        .map(|gen| self.emit_type(gen))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<{}>", generics_ty_str)
                } else {
                    String::new()
                };

                let args_str = args
                    .iter()
                    .map(|arg| self.emit_value(arg))
                    .collect::<Vec<_>>()
                    .join(", ");

                return format!("{}{}({})", callee_str, generics_str, args_str);
            }

            _ => String::new(),
        }
    }

    fn emit_member(&self, exprs: &[Expr]) -> String {
        exprs
            .iter()
            .map(|e| self.emit_value(e))
            .collect::<Vec<_>>()
            .join(".")
    }

    fn emit_type(&self, ty: &Expr) -> String {
        match ty {
            Expr::TypedSymbol(type_name) => type_name.clone(),
            Expr::TypedVariant(lhs, rhs) => self.emit_variant(lhs, rhs),

            Expr::TypedLiteral(type_name) => {
                format!("decltype({})", self.emit_value(type_name))
            }

            Expr::TypedSymbolGeneric(type_name, ty_params) => {
                let ty_params_str = if !ty_params.is_empty() {
                    let ty_params_str = ty_params
                        .iter()
                        .map(|gen| self.emit_type(gen))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<{}>", ty_params_str)
                } else {
                    String::new()
                };

                format!("{}{}", type_name, ty_params_str)
            }

            _ => String::new(),
        }
    }

    fn emit_variant(&self, lhs: &Expr, rhs: &Expr) -> String {
        let lhs_str = self.emit_type(lhs);

        match rhs {
            // match against RHS typed variants which is the nesting form
            // based on the parsing rule, expecting a LHS and RHS
            Expr::TypedVariant(next_lhs, next_rhs) => {
                let rhs_str = self.emit_variant(next_lhs, next_rhs);
                format!("{} | {}", lhs_str, rhs_str)
            }
            _ => {
                let rhs_str = self.emit_type(rhs);
                format!("variant<{} | {}>", lhs_str, rhs_str)
            }
        }
    }
}
