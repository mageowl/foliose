use lib::{
    error::Result,
    instruction::{Comparison, Reporter},
    span::{Chunk, Span},
};

use crate::parser::expression::{Expression, Operation, UnaryOperation};

use super::{Compile, CompileChunk, CompilerScope};

impl<'a> Compile<'a> for Expression<'a> {
    type Output = Reporter<'a>;
    fn compile(self, span: Span, scope: &mut CompilerScope<'a, '_>) -> Result<Chunk<Self::Output>> {
        match self {
            Expression::Null => Ok(Chunk::new(Reporter::Null, span)),
            Expression::String(str) => Ok(Chunk::new(Reporter::ConstStr(str), span)),
            Expression::Int(int) => Ok(Chunk::new(Reporter::ConstInt(int), span)),
            Expression::Float(float) => Ok(Chunk::new(Reporter::ConstFloat(float), span)),
            Expression::Boolean(bool) => Ok(Chunk::new(Reporter::ConstBool(bool), span)),
            Expression::Array(items) => Ok(Chunk::new(
                Reporter::Array(Chunk::new(
                    items
                        .into_iter()
                        .map(|c| c.compile(scope))
                        .collect::<Result<_>>()?,
                    span,
                )),
                span,
            )),
            Expression::Function { parameters, body } => {
                let mut scope = CompilerScope::new(Some(scope));
                scope.variables.extend(parameters.iter().map(|p| p.data));

                Ok(Chunk::new(
                    Reporter::Function {
                        parameters,
                        body: body.unbox().compile(&mut scope)?.as_box(),
                    },
                    span,
                ))
            }
            // TODO: check for variable in scope
            Expression::Variable(name) => Ok(Chunk::new(
                Reporter::Get {
                    map: Chunk::new(Reporter::Parent(scope.get_var(name).unwrap_or(0)), span)
                        .as_box(),
                    name: Chunk::new(name, span),
                },
                span,
            )),
            Expression::GetProp(map, name) => Ok(Chunk::new(
                Reporter::Get {
                    map: map.unbox().compile(scope)?.as_box(),
                    name,
                },
                span,
            )),
            Expression::DynProp(map, attr) => Ok(Chunk::new(
                Reporter::DynGet {
                    map: map.unbox().compile(scope)?.as_box(),
                    attr: attr.unbox().compile(scope)?.as_box(),
                },
                span,
            )),
            Expression::Call { value, args } => Ok(Chunk::new(
                Reporter::Call(
                    value.unbox().compile(scope)?.as_box(),
                    args.into_iter()
                        .map(|c| c.compile(scope))
                        .collect::<Result<_>>()?,
                ),
                span,
            )),
            Expression::Block(block) => Ok(Chunk::new(
                Reporter::Block(block.compile(span, scope)?.data),
                span,
            )),
            Expression::If { blocks, else_block } => Ok(Chunk::new(
                Reporter::If {
                    blocks: blocks
                        .into_iter()
                        .map(|(cond, body)| Ok((cond.compile(scope)?, body.compile(scope)?)))
                        .collect::<Result<_>>()?,
                    else_block: else_block
                        .map(|b| Ok(b.unbox().compile(scope)?.as_box()))
                        .transpose()?,
                },
                span,
            )),
            Expression::BinaryOp { a, b, op } => {
                let a = a.unbox().compile(scope)?.as_box();
                let b = b.unbox().compile(scope)?.as_box();
                Ok(Chunk::new(
                    match op.data {
                        Operation::Concat => Reporter::Concat { a, b },
                        Operation::Exponent => Reporter::Exponent { a, b },
                        Operation::Multiply => Reporter::Multiply { a, b },
                        Operation::Divide => Reporter::Divide { a, b },
                        Operation::Add => Reporter::Add { a, b },
                        Operation::Subtract => Reporter::Subtract { a, b },
                        Operation::Equals => Reporter::Equality { a, b },
                        Operation::NotEquals => {
                            Reporter::Not(Chunk::new(Reporter::Equality { a, b }, span).as_box())
                        }
                        Operation::Lt => Reporter::Inequality {
                            a,
                            b,
                            op: Chunk::new(Comparison::LessThan, op.span),
                        },
                        Operation::LtEqual => Reporter::Not(
                            Chunk::new(
                                Reporter::Inequality {
                                    a,
                                    b,
                                    op: Chunk::new(Comparison::GreaterThan, op.span),
                                },
                                span,
                            )
                            .as_box(),
                        ),
                        Operation::Gt => Reporter::Inequality {
                            a,
                            b,
                            op: Chunk::new(Comparison::GreaterThan, op.span),
                        },
                        Operation::GtEqual => Reporter::Not(
                            Chunk::new(
                                Reporter::Inequality {
                                    a,
                                    b,
                                    op: Chunk::new(Comparison::LessThan, op.span),
                                },
                                span,
                            )
                            .as_box(),
                        ),
                        Operation::And => Reporter::And { a, b },
                        Operation::Or => Reporter::Or { a, b },
                    },
                    span,
                ))
            }
            Expression::UnaryOp { a, op } => Ok(match op.data {
                UnaryOperation::Not => {
                    Chunk::new(Reporter::Not(a.unbox().compile(scope)?.as_box()), span)
                }
                UnaryOperation::Negative => {
                    Chunk::new(Reporter::Negative(a.unbox().compile(scope)?.as_box()), span)
                }
            }),
            Expression::Group(expr) => expr.compile(span, scope),
        }
    }
}
