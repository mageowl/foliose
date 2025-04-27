use lib::{
    error::{Error, Result},
    instruction::{Instruction, Reporter},
    span::{Chunk, Span},
};

use crate::parser::{
    expression::Expression,
    statement::{AssignOperator, Statement},
};

use super::{Compile, CompileChunk, CompilerScope};

impl<'a> Compile<'a> for Statement<'a> {
    type Output = Instruction<'a>;
    fn compile(
        self,
        span: Span,
        scope: &mut CompilerScope<'a, '_>,
    ) -> Result<Chunk<Instruction<'a>>> {
        match self {
            Self::Assign {
                name: name_expr,
                op,
                value,
            } => {
                let (name, map) = match name_expr.data {
                    // TODO: check parent scopes
                    Expression::Variable(name) => {
                        let up = scope.get_var(name);
                        if up.is_none() {
                            scope.variables.push(name);
                        }
                        (
                            Chunk::new(name, name_expr.span),
                            Chunk::new(Reporter::Parent(up.unwrap_or(0)), name_expr.span),
                        )
                    }
                    // TODO: generate objects
                    Expression::GetProp(expr, prop) => (prop, expr.unbox().compile(scope)?),
                    _ => {
                        return Err(Error::new(
                            "Only variables and properties can be assigned to.",
                            name_expr.span,
                        ));
                    }
                };

                let value = value.compile(scope)?;
                let value_span = value.span;
                let value = match op.data {
                    AssignOperator::Set => value,
                    AssignOperator::Add => Chunk::new(
                        Reporter::Add {
                            a: Chunk::new(
                                Reporter::Get {
                                    map: map.clone().as_box(),
                                    name,
                                },
                                name_expr.span,
                            )
                            .as_box(),
                            b: value.as_box(),
                        },
                        value_span,
                    ),
                    AssignOperator::Subtract => Chunk::new(
                        Reporter::Subtract {
                            a: Chunk::new(
                                Reporter::Get {
                                    map: map.clone().as_box(),
                                    name,
                                },
                                name_expr.span,
                            )
                            .as_box(),
                            b: value.as_box(),
                        },
                        value_span,
                    ),
                    AssignOperator::Multiply => Chunk::new(
                        Reporter::Multiply {
                            a: Chunk::new(
                                Reporter::Get {
                                    map: map.clone().as_box(),
                                    name,
                                },
                                name_expr.span,
                            )
                            .as_box(),
                            b: value.as_box(),
                        },
                        value_span,
                    ),
                    AssignOperator::Divide => Chunk::new(
                        Reporter::Divide {
                            a: Chunk::new(
                                Reporter::Get {
                                    map: map.clone().as_box(),
                                    name,
                                },
                                name_expr.span,
                            )
                            .as_box(),
                            b: value.as_box(),
                        },
                        value_span,
                    ),
                };

                Ok(Chunk::new(Instruction::Set { map, name, value }, span))
            }
            Self::Expr(expression) => Ok(Chunk::new(
                Instruction::Void(expression.compile(span, scope)?.data),
                span,
            )),
            Self::While { cond, body } => Ok(Chunk::new(
                Instruction::While {
                    condition: cond.compile(scope)?,
                    body: body.compile(scope)?,
                },
                span,
            )),
            Self::For { name, iter, body } => Ok(Chunk::new(
                Instruction::For {
                    name,
                    iter: iter.compile(scope)?,
                    body: body.compile(scope)?,
                },
                span,
            )),
            Self::Return(value) => Ok(Chunk::new(Instruction::Return(value.compile(scope)?), span)),
        }
    }
}
