use crate::parser::{BinOp, BitOp, CompareOp, Expr, UnaryOp};
use crate::visit::{EsperContext, Visitor};

// note: esper outputs with some non-practical patterns:
// 1 - where GLIBXX is not defined or not in /usr/include/c++, we conditionally
//     include libstdc++ headers since we will only be compiling with clang++
// 2 - using namespace std is forced since the :: operator is reserved (lst slice)
// 3 - public class member definitions (leaky abstractions) is forced
// 4 - C++ initializer list for RHS list-like expressions

#[derive(Debug, Clone)]
pub struct EmitContextImpl {
    pub level: usize,
    pub output: String,
    pub module_id: String,
    pub use_prelude: bool,
}

impl EmitContextImpl {
    pub fn new() -> Self {
        EmitContextImpl {
            level: 0,
            output: String::new(),
            module_id: String::new(),
            use_prelude: false,
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

        ctx.module_id = module_id.into();
        self.emit_expr(&mut ctx, expr);
        ctx.output
    }

    pub fn emit_expr(&self, ctx: &mut EmitContextImpl, expr: &Expr) {
        match expr {
            Expr::Program(exprs) => {
                if ctx.use_prelude {
                    ctx.emit(include_str!("./prelude.h"));
                }

                ctx.emit("using namespace std;\n");
                ctx.emit(&format!("namespace {} {{", ctx.module_id));
                // ctx.level = 2;
                // ctx.emit("");

                for sub_expr in exprs {
                    self.emit_expr(ctx, sub_expr);
                }

                // ctx.emit("");
                ctx.emit(&format!("}} // namespace {}", ctx.module_id));
                ctx.emit("");
                ctx.emit(&format!(
                    "int main(int argc, const char** argv) {{ return {}::main(argc, std::vector<std::string>(argv + 1, argv + argc)); }}",
                    ctx.module_id)
                );
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
                                    | Expr::Pass
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
                                    | Expr::Pass
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

            Expr::Match(cond, cases) => {
                let cond_str = self.emit_value(cond);
                let indent = ctx.indent();

                ctx.emit(&format!("{}std::visit([](auto&& _) {{", indent));
                ctx.level += 2;
                let indent = ctx.indent();
                ctx.emit(&format!("{}using T = std::decay_t<decltype(_)>;", indent));

                // ctx.level += 2;

                for (pat, body) in cases {
                    let indent = ctx.indent();
                    let pat_str = self.emit_value(&Expr::Var(pat.clone()));

                    ctx.emit(&format!(
                        "{}if constexpr (std::is_same_v<T, {}>) {{",
                        indent, pat_str
                    ));
                    ctx.level += 2;

                    for expr in body {
                        self.emit_expr(ctx, expr);
                    }

                    ctx.level -= 2;
                    ctx.emit(&format!("{}}}", indent));
                }

                ctx.level -= 2;
                ctx.emit(&format!("{}}}, {});", indent, cond_str));
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

            Expr::TypeAlias(name, ty_params, rhs) => {
                let indent = ctx.indent();

                let template_str = if ty_params.is_empty() {
                    String::new()
                } else {
                    let ty_params_str = ty_params
                        .iter()
                        .map(|ty| format!("typename {}", self.emit_type(ty)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("template<{}> ", ty_params_str)
                };

                match rhs.as_ref() {
                    Expr::TypedSymbol(ty) => {
                        ctx.emit(&format!(
                            "{}{}using {} = {};",
                            indent, template_str, name, ty
                        ));
                    }

                    Expr::TypedRecord(record_expr) => {
                        if let Expr::Record(entries) = record_expr.as_ref() {
                            ctx.emit(&format!("{}{}struct {} {{", indent, template_str, name));
                            ctx.level += 2;

                            for entry in entries {
                                if entry.len() == 2 {
                                    let indent = ctx.indent();
                                    let key_str = self.emit_value(&entry[0]);
                                    let value_str = self.emit_type(&entry[1]);

                                    ctx.emit(&format!(
                                        "{}using {} = {};",
                                        indent, key_str, value_str
                                    ));
                                }
                            }

                            ctx.level -= 2;
                            ctx.emit(&format!("{}}};", indent));
                        }
                    }

                    _ => {
                        ctx.emit(&format!(
                            "{}{}using {} = {};",
                            indent,
                            template_str,
                            name,
                            self.emit_type(rhs)
                        ));
                    }
                }
            }

            Expr::Directive(directive, expr) => {
                if let Expr::Call(callee, args) = directive.as_ref() {
                    if let Expr::Var(directive_name) = callee.as_ref() {
                        if directive_name == "extend" {
                            if let [Expr::Var(ident), Expr::Var(ty)] = &args[..] {
                                self.emit_extend(ctx, ident, ty, expr);
                            }
                        }
                    }
                } else {
                    // only go through modifiers that are non-call exprs
                    let out = self.emit_value(&Expr::Directive(
                        Box::new(directive.as_ref().to_owned()),
                        Box::new(expr.as_ref().to_owned()),
                    ));

                    if matches!(
                        **expr,
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
                            | Expr::Pass
                            | Expr::TypedCall(_, _, _)
                    ) {
                        // modifiers on expressions (emit_value)
                        let indent = ctx.indent();
                        ctx.emit(&format!("{}{};", indent, out));
                    } else {
                        // modifiers on statements (emit_expr)
                        let indent = ctx.indent();
                        ctx.emit(&format!("{}{}", indent, out));
                        self.emit_expr(ctx, expr);
                    }
                }
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
                    CompareOp::Eq => "==",
                    CompareOp::Neq => "!=",
                    CompareOp::And => "&&",
                    CompareOp::Or => "||",
                };

                format!("({} {} {})", lhs_str, op_str, rhs_str)
            }

            Expr::Unary(expr, op) => {
                let expr_str = self.emit_value(expr);
                let op_str = match op {
                    UnaryOp::Ref => "&",
                    UnaryOp::Deref => "*",
                    UnaryOp::BitNot => "~",
                };

                format!("{}{}", op_str, expr_str)
            }

            Expr::Bit(lhs, op, rhs) => {
                if matches!(&op, BitOp::Rotl | BitOp::Rotr) {
                    let lhs_str = self.emit_value(lhs);
                    let rhs_str = self.emit_value(rhs);
                    let op_str = match op {
                        BitOp::Rotl => "__builtin_rotateleft32",
                        BitOp::Rotr => "__builtin_rotateright32",
                        _ => "",
                    };

                    format!("{}({}, {})", op_str, lhs_str, rhs_str)
                } else {
                    let lhs_str = self.emit_value(lhs);
                    let rhs_str = self.emit_value(rhs);
                    let op_str = match op {
                        BitOp::Shl => "<<",
                        BitOp::Shr => ">>",
                        BitOp::And => "&",
                        BitOp::Or => "|",
                        BitOp::Xor => "^",
                        _ => "",
                    };

                    format!("({} {} {})", lhs_str, op_str, rhs_str)
                }
            }

            Expr::Range(lhs, rhs) => {
                let lhs_str = self.emit_value(lhs);
                let rhs_str = self.emit_value(rhs);

                format!("views::iota({}, {})", lhs_str, rhs_str)
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

            Expr::Directive(directive, expr) => {
                if let Expr::Var(directive_name) = directive.as_ref() {
                    let mut specifier = String::new();

                    match directive_name.as_str() {
                        "const" => specifier = "constexpr".to_string(),
                        "static" => specifier = "static".to_string(),
                        "inline" => specifier = "inline".to_string(),
                        _ => {}
                    }

                    if !specifier.is_empty() {
                        if matches!(**expr, Expr::Directive(_, _)) {
                            return format!("{}", specifier);
                        } else {
                            let value = self.emit_value(expr);
                            return format!("{} {}", specifier, value);
                        }
                    }
                }

                String::new()
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

            Expr::TypedUnary(expr) => {
                if let Expr::Unary(ty_expr, op) = &**expr {
                    return format!(
                        "{}{}",
                        self.emit_type(&*ty_expr),
                        match op {
                            UnaryOp::Ref => "&",
                            UnaryOp::Deref => "*",
                            _ => "",
                        },
                    );
                }

                format!("")
            }

            Expr::TypedLiteral(type_name) => {
                format!("decltype({})", self.emit_value(type_name))
            }

            Expr::TypedOptional(ty) => {
                format!("optional<{}>", self.emit_type(ty))
            }

            Expr::TypedMember(member_expr) => self.emit_value(member_expr).replace(".", "::"),

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

            Expr::TypedFn(fn_expr) => {
                if let Expr::Fn(params, body) = fn_expr.as_ref() {
                    if let Some(last_expr) = body.last() {
                        let params_str = params
                            .iter()
                            .map(|(_, ty)| {
                                ty.as_ref()
                                    .map(|t| self.emit_type(t))
                                    .unwrap_or_else(|| "void".to_string())
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        let ret_type_str = self.emit_type(last_expr);

                        return format!("std::function<{}({})>", ret_type_str, params_str);
                    }
                }

                String::new()
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
                format!("variant<{}, {}>", lhs_str, rhs_str)
            }
        }
    }

    fn emit_extend(&self, ctx: &mut EmitContextImpl, ident: &str, ext_ty: &str, expr: &Expr) {
        match expr {
            Expr::TypeAlias(name, ty_params, rhs) => {
                // we'll only modify the RHS type (not the generic type params) based on @extend
                // let updated_rhs = self.replace_with_enable_if(ident, ext_ty, rhs);
                let updated_rhs = match rhs.as_ref() {
                    Expr::TypedSymbol(type_name) if type_name == ident => {
                        Box::new(Expr::TypedSymbol(format!(
                            "std::enable_if_t<std::is_same<{}, {}>::value, {}>",
                            ident, ext_ty, ident
                        )))
                    }

                    _ => rhs.clone(),
                };

                self.emit_expr(
                    ctx,
                    &Expr::TypeAlias(name.clone(), ty_params.clone(), updated_rhs),
                );
            }

            // recurse all nested extend directives
            Expr::Directive(directive, inner_expr) => {
                self.emit_extend(ctx, ident, ext_ty, inner_expr);
                // self.emit_expr(ctx, expr);
            }

            _ => {}
        }
    }
}
