use crate::parser::*;

pub trait EsperContext {
    fn new() -> Self
    where
        Self: Sized;
}

pub trait Visitor {
    fn visit(
        &self,
        ctx: &mut dyn EsperContext,
        callback: &mut dyn FnMut(&mut dyn EsperContext, &Expr),
    );
}

// we will only visit expressions that branch or wrap
// expressions in the variant definitions, the callback
// won't be return based so EmitContextImpl should allow
// emitting from the callback into different outputs

impl Visitor for Expr {
    fn visit(
        &self,
        ctx: &mut dyn EsperContext,
        callback: &mut dyn FnMut(&mut dyn EsperContext, &Expr),
    ) {
        callback(ctx, self);

        match self {
            Expr::Program(exprs) => {
                for expr in exprs {
                    expr.visit(ctx, callback);
                }
            }

            Expr::Let(_, expr) => expr.visit(ctx, callback),

            Expr::Assign(lhs, rhs) => {
                lhs.visit(ctx, callback);
                rhs.visit(ctx, callback);
            }

            Expr::Var(_) => {}

            Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Char(_) | Expr::String(_) => {}

            Expr::List(exprs) => {
                for expr in exprs {
                    expr.visit(ctx, callback);
                }
            }

            Expr::Record(entries) => {
                for entry in entries {
                    for expr in entry {
                        expr.visit(ctx, callback);
                    }
                }
            }

            Expr::Range(start, end) => {
                start.visit(ctx, callback);
                end.visit(ctx, callback);
            }

            Expr::Bin(lhs, _, rhs) => {
                lhs.visit(ctx, callback);
                rhs.visit(ctx, callback);
            }

            Expr::Compare(lhs, _, rhs) => {
                lhs.visit(ctx, callback);
                rhs.visit(ctx, callback);
            }

            Expr::Bit(lhs, _, rhs) => {
                lhs.visit(ctx, callback);
                rhs.visit(ctx, callback);
            }

            Expr::Unary(expr, _) => {
                expr.visit(ctx, callback);
            }

            Expr::If(cond, then_body, else_body) => {
                cond.visit(ctx, callback);

                for expr in then_body {
                    expr.visit(ctx, callback);
                }

                if let Some(else_body) = else_body {
                    for expr in else_body {
                        expr.visit(ctx, callback);
                    }
                }
            }

            Expr::Loop(var, iter, body) => {
                var.visit(ctx, callback);
                iter.visit(ctx, callback);

                for expr in body {
                    expr.visit(ctx, callback);
                }
            }

            Expr::Match(cond, cases) => {
                cond.visit(ctx, callback);

                for (pat, body) in cases {
                    for expr in body {
                        expr.visit(ctx, callback);
                    }
                }
            }

            Expr::Fn(_, body) => {
                for expr in body {
                    expr.visit(ctx, callback);
                }
            }

            Expr::Member(exprs) => {
                for expr in exprs {
                    expr.visit(ctx, callback);
                }
            }

            Expr::Call(callee, args) => {
                callee.visit(ctx, callback);

                for arg in args {
                    arg.visit(ctx, callback);
                }
            }

            Expr::Struct(_, entries) => {
                for (_, expr) in entries {
                    expr.visit(ctx, callback);
                }
            }

            Expr::TypedSymbol(_) => {}

            Expr::TypedSymbolGeneric(_, exprs) => {
                for expr in exprs {
                    expr.visit(ctx, callback);
                }
            }

            Expr::TypedLiteral(expr) => expr.visit(ctx, callback),

            Expr::TypedMember(expr) => expr.visit(ctx, callback),
            Expr::TypedOptional(expr) => expr.visit(ctx, callback),

            Expr::TypedVariant(lhs, rhs) => {
                lhs.visit(ctx, callback);
                rhs.visit(ctx, callback);
            }

            Expr::TypedFn(expr) => expr.visit(ctx, callback),

            Expr::TypedLet(_, ty, expr) => {
                ty.visit(ctx, callback);
                expr.visit(ctx, callback);
            }

            Expr::TypedRecord(record) => {
                if let Expr::Record(record) = *record.to_owned() {
                    for entry in record.clone() {
                        for expr in entry {
                            expr.visit(ctx, callback);
                        }
                    }
                }
            }

            Expr::TypeAlias(_, _, expr) => expr.visit(ctx, callback),

            Expr::TypedCall(callee, _, args) => {
                callee.visit(ctx, callback);
                for arg in args {
                    arg.visit(ctx, callback);
                }
            }

            Expr::Directive(directive, expr) => {
                directive.visit(ctx, callback);
                expr.visit(ctx, callback);
            }
        }
    }
}
