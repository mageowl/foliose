use std::slice::Iter;

use crate::span::Chunk;
use self_cell::self_cell;

use super::{Instruction, Reporter};

type Dependent<'a> = Chunk<Reporter<'a>>;
self_cell!(
    struct Inner {
        owner: Vec<String>,
        #[covariant]
        dependent: Dependent,
    }
);

pub struct OwnedReporter {
    inner: Inner,
}

fn visit_instruction(owned_buf: &mut Vec<String>, instruction: &Instruction) {
    match instruction {
        Instruction::Set { map, name, value } => {
            visit_reporter(owned_buf, &map.data);
            owned_buf.push(name.data.to_string());
            visit_reporter(owned_buf, &value.data);
        }
        Instruction::While { condition, body } => {
            visit_reporter(owned_buf, &condition.data);
            for inst in &body.data {
                visit_instruction(owned_buf, &inst.data);
            }
        }
        Instruction::For { name, iter, body } => {
            owned_buf.push(name.data.to_string());
            visit_reporter(owned_buf, &iter.data);
            for inst in &body.data {
                visit_instruction(owned_buf, &inst.data);
            }
        }
        Instruction::Return(value) => visit_reporter(owned_buf, &value.data),
        Instruction::Void(reporter) => visit_reporter(owned_buf, &reporter),
    }
}
fn visit_reporter(owned_buf: &mut Vec<String>, reporter: &Reporter) {
    match reporter {
        Reporter::Parent(_) => (),
        Reporter::Null => (),
        Reporter::ConstStr(str) => owned_buf.push(str.to_string()),
        Reporter::ConstInt(_) => (),
        Reporter::ConstFloat(_) => (),
        Reporter::ConstBool(_) => (),
        Reporter::Block(body) => {
            for inst in body {
                visit_instruction(owned_buf, &inst.data);
            }
        }
        Reporter::Array(items) => {
            for item in &items.data {
                visit_reporter(owned_buf, &item.data);
            }
        }
        Reporter::Function { parameters, body } => {
            for parameter in parameters {
                owned_buf.push(parameter.data.to_string());
            }
            visit_reporter(owned_buf, &body.data);
        }
        Reporter::Get { map, name } => {
            visit_reporter(owned_buf, &map.data);
            owned_buf.push(name.data.to_string());
        }
        Reporter::DynGet { map, attr } => {
            visit_reporter(owned_buf, &map.data);
            visit_reporter(owned_buf, &attr.data);
        }
        Reporter::Call(func, args) => {
            visit_reporter(owned_buf, &func.data);
            for arg in args {
                visit_reporter(owned_buf, &arg.data);
            }
        }
        Reporter::If { blocks, else_block } => {
            for (cond, body) in blocks {
                visit_reporter(owned_buf, &cond.data);
                visit_reporter(owned_buf, &body.data);
            }
            if let Some(body) = else_block {
                visit_reporter(owned_buf, &body.data);
            }
        }
        Reporter::Add { a, b }
        | Reporter::Subtract { a, b }
        | Reporter::Multiply { a, b }
        | Reporter::Divide { a, b }
        | Reporter::Exponent { a, b }
        | Reporter::Concat { a, b }
        | Reporter::And { a, b }
        | Reporter::Or { a, b }
        | Reporter::Equality { a, b }
        | Reporter::Inequality { a, b, op: _ } => {
            visit_reporter(owned_buf, &a.data);
            visit_reporter(owned_buf, &b.data);
        }
        Reporter::Not(value) | Reporter::Negative(value) => visit_reporter(owned_buf, &value.data),
    }
}
fn build_instruction<'a, 'b>(
    owned_buf: &mut Iter<'b, String>,
    instruction: Chunk<Instruction<'a>>,
) -> Chunk<Instruction<'b>> {
    Chunk::new(
        match instruction.data {
            Instruction::Set { map, name, value } => Instruction::Set {
                map: build_reporter(owned_buf, map),
                name: Chunk::new(owned_buf.next().unwrap().as_str(), name.span),
                value: build_reporter(owned_buf, value),
            },
            Instruction::While { condition, body } => Instruction::While {
                condition: build_reporter(owned_buf, condition),
                body: Chunk::new(
                    body.data
                        .into_iter()
                        .map(|inst| build_instruction(owned_buf, inst))
                        .collect(),
                    body.span,
                ),
            },
            Instruction::For { name, iter, body } => Instruction::For {
                name: Chunk::new(owned_buf.next().unwrap().as_str(), name.span),
                iter: build_reporter(owned_buf, iter),
                body: Chunk::new(
                    body.data
                        .into_iter()
                        .map(|inst| build_instruction(owned_buf, inst))
                        .collect(),
                    body.span,
                ),
            },
            Instruction::Return(value) => Instruction::Return(build_reporter(owned_buf, value)),
            Instruction::Void(reporter) => Instruction::Void(
                build_reporter(owned_buf, Chunk::new(reporter, instruction.span)).data,
            ),
        },
        instruction.span,
    )
}
fn build_reporter<'a, 'b>(
    owned_buf: &mut Iter<'b, String>,
    reporter: Chunk<Reporter<'a>>,
) -> Chunk<Reporter<'b>> {
    Chunk::new(
        match reporter.data {
            Reporter::Parent(up) => Reporter::Parent(up),
            Reporter::Null => Reporter::Null,
            Reporter::ConstStr(_) => Reporter::ConstStr(owned_buf.next().unwrap().as_str()),
            Reporter::ConstInt(int) => Reporter::ConstInt(int),
            Reporter::ConstFloat(float) => Reporter::ConstFloat(float),
            Reporter::ConstBool(bool) => Reporter::ConstBool(bool),
            Reporter::Block(body) => Reporter::Block(
                body.into_iter()
                    .map(|inst| build_instruction(owned_buf, inst))
                    .collect(),
            ),
            Reporter::Array(items) => Reporter::Array(Chunk::new(
                items
                    .data
                    .into_iter()
                    .map(|reporter| build_reporter(owned_buf, reporter))
                    .collect(),
                items.span,
            )),
            Reporter::Function { parameters, body } => Reporter::Function {
                parameters: owned_buf
                    .zip(parameters)
                    .map(|(o, p)| Chunk::new(o.as_str(), p.span))
                    .collect(),
                body: build_reporter(owned_buf, body.unbox()).as_box(),
            },
            Reporter::Get { map, name } => Reporter::Get {
                map: build_reporter(owned_buf, map.unbox()).as_box(),
                name: Chunk::new(owned_buf.next().unwrap(), name.span),
            },
            Reporter::DynGet { map, attr } => Reporter::DynGet {
                map: build_reporter(owned_buf, map.unbox()).as_box(),
                attr: build_reporter(owned_buf, attr.unbox()).as_box(),
            },
            Reporter::Call(func, args) => Reporter::Call(
                build_reporter(owned_buf, func.unbox()).as_box(),
                args.into_iter()
                    .map(|arg| build_reporter(owned_buf, arg))
                    .collect(),
            ),
            Reporter::If { blocks, else_block } => Reporter::If {
                blocks: blocks
                    .into_iter()
                    .map(|(cond, body)| {
                        (
                            build_reporter(owned_buf, cond),
                            build_reporter(owned_buf, body),
                        )
                    })
                    .collect(),
                else_block: else_block.map(|body| build_reporter(owned_buf, body.unbox()).as_box()),
            },
            Reporter::Add { a, b } => Reporter::Add {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Subtract { a, b } => Reporter::Subtract {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Multiply { a, b } => Reporter::Multiply {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Divide { a, b } => Reporter::Divide {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Exponent { a, b } => Reporter::Exponent {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Concat { a, b } => Reporter::Concat {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::And { a, b } => Reporter::And {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Or { a, b } => Reporter::Or {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Equality { a, b } => Reporter::Equality {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
            },
            Reporter::Inequality { a, b, op } => Reporter::Inequality {
                a: build_reporter(owned_buf, a.unbox()).as_box(),
                b: build_reporter(owned_buf, b.unbox()).as_box(),
                op,
            },
            Reporter::Not(value) => {
                Reporter::Not(build_reporter(owned_buf, value.unbox()).as_box())
            }
            Reporter::Negative(value) => {
                Reporter::Negative(build_reporter(owned_buf, value.unbox()).as_box())
            }
        },
        reporter.span,
    )
}

impl OwnedReporter {
    pub fn new(reporter: Chunk<Reporter>) -> Self {
        let mut owned_buf = Vec::new();
        visit_reporter(&mut owned_buf, &reporter.data);

        Self {
            inner: Inner::new(owned_buf, |buf| {
                let mut iter = buf.iter();
                build_reporter(&mut iter, reporter)
            }),
        }
    }

    pub fn borrow(&self) -> &Chunk<Reporter> {
        self.inner.borrow_dependent()
    }
}
