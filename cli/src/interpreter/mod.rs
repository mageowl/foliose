use std::{cell::RefCell, collections::HashMap, rc::Rc};

use function::Function;
use lib::{
    error::{Error, Result},
    instruction::{Comparison, Instruction, Reporter},
    span::Chunk,
    type_error,
    value::{MapRef, Value},
};

mod function;

#[derive(Debug)]
pub struct Scope {
    parent: Option<Rc<RefCell<dyn MapRef>>>,
    variables: HashMap<String, Value>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<dyn MapRef>>>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
        }
    }
    fn up(rc: &Rc<RefCell<Scope>>, i: usize) -> Option<Rc<RefCell<dyn MapRef>>> {
        fn up_dyn(rc: &Rc<RefCell<dyn MapRef>>, i: usize) -> Option<Rc<RefCell<dyn MapRef>>> {
            if i == 0 {
                Some(rc.clone())
            } else {
                rc.borrow().parent().as_ref().and_then(|p| up_dyn(p, i - 1))
            }
        }
        if i == 0 {
            Some(rc.clone())
        } else {
            rc.borrow().parent().as_ref().and_then(|p| up_dyn(p, i - 1))
        }
    }
}
impl MapRef for Scope {
    fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
    fn set(&mut self, name: String, val: Value) {
        self.variables.insert(name, val);
    }
    fn parent(&self) -> Option<Rc<RefCell<dyn MapRef>>> {
        self.parent.clone()
    }
    fn as_hashmap(&self) -> Option<&HashMap<String, Value>> {
        Some(&self.variables)
    }
}

pub fn run_block(
    scope: &Rc<RefCell<Scope>>,
    instructions: Vec<Chunk<Instruction<'_>>>,
) -> Result<Value> {
    let mut return_value = None;
    for instruction in instructions {
        match instruction.data {
            Instruction::Set { map, name, value } => {
                let map = evaluate(scope, map)?;
                let value = evaluate(scope, value)?;
                match map {
                    Value::MapRef(rc) => rc.borrow_mut().set(name.data.to_string(), value),
                    Value::Map(_) => (), // its useless to insert a item into an owned map.
                    _ => todo!(),
                };
            }
            Instruction::While { condition, body } => {
                while match evaluate(scope, condition.clone())? {
                    Value::Boolean(b) => b,
                    value => {
                        return Err(Error::new(
                            type_error!("boolean", value.type_of()),
                            condition.span,
                        ));
                    }
                } {
                    let new_scope = Rc::new(RefCell::new(Scope::new(Some(scope.clone()))));
                    run_block(&new_scope, body.data.clone())?;
                }
            }
            Instruction::For { name, iter, body } => {
                let iter_span = iter.span;
                let iter = evaluate(scope, iter)?;
                let mut next: Box<dyn FnMut() -> Result<Value>> = match iter {
                    Value::Function(callable) => {
                        Box::new(move || callable.call(Vec::new(), iter_span))
                    }
                    Value::Array(items) => {
                        let mut iter = items.into_iter();
                        Box::new(move || Ok(iter.next().unwrap_or(Value::Null)))
                    }
                    _ => {
                        return Err(Error::new(
                            format!(
                                "Expected an iterator function, but instead got {}",
                                iter.primative_type()
                            ),
                            iter_span,
                        ));
                    }
                };
                let mut item = next()?;
                let name = name.data;
                let mut key = name.to_string();
                while !matches!(item, Value::Null) {
                    let new_scope = Rc::new(RefCell::new(Scope::new(Some(scope.clone()))));
                    new_scope.borrow_mut().variables.insert(key, item);
                    run_block(&new_scope, body.data.clone())?;
                    item = next()?;
                    key = new_scope
                        .borrow_mut()
                        .variables
                        .remove_entry(name)
                        .unwrap()
                        .0;
                }
            }
            Instruction::Return(reporter) => {
                return_value = Some(evaluate(scope, reporter)?);
                break;
            }
            Instruction::Void(reporter) => {
                evaluate(scope, Chunk::new(reporter, instruction.span))?;
            }
        }
    }
    Ok(return_value.unwrap_or_else(move || Value::MapRef(scope.clone())))
}

pub fn evaluate(scope: &Rc<RefCell<Scope>>, reporter: Chunk<Reporter<'_>>) -> Result<Value> {
    match reporter.data {
        Reporter::Parent(up) => Ok(Value::MapRef(Scope::up(scope, up).ok_or_else(|| {
            dbg!(scope);
            dbg!(up);
            Error::new("Failed to get parent scope", reporter.span)
        })?)),
        Reporter::Null => Ok(Value::Null),
        Reporter::ConstStr(str) => Ok(Value::String(str.to_string())),
        Reporter::ConstInt(int) => Ok(Value::Int(int)),
        Reporter::ConstFloat(float) => Ok(Value::Float(float)),
        Reporter::ConstBool(bool) => Ok(Value::Boolean(bool)),
        Reporter::Block(chunks) => {
            let scope = Scope::new(Some(scope.clone()));
            run_block(&Rc::new(RefCell::new(scope)), chunks)
        }
        Reporter::Array(items) => Ok(Value::Array(
            items
                .data
                .into_iter()
                .map(|c| evaluate(scope, c))
                .collect::<Result<_>>()?,
        )),
        Reporter::Function { parameters, body } => Ok(Value::Function(Rc::new(Function::new(
            scope.clone(),
            parameters.iter().map(|s| s.data.to_string()).collect(),
            body.unbox(),
        )))),
        Reporter::Get { map, name } => {
            let map_span = map.span;
            match evaluate(scope, map.unbox())? {
                Value::Map(map) => map.get(name.data).cloned(),
                Value::MapRef(map) => map.borrow().get(name.data).cloned(),
                v => return Err(Error::new(type_error!("map", v.type_of()), map_span)),
            }
            .ok_or_else(|| Error::new(format!("No property '{}'.", name.data), name.span))
        }
        Reporter::DynGet { map, attr } => {
            let map_span = map.span;
            let attr_span = attr.span;
            let name = match evaluate(scope, attr.unbox())? {
                Value::String(str) => str,
                v => return Err(Error::new(type_error!("string", v.type_of()), attr_span)),
            };
            match evaluate(scope, map.unbox())? {
                Value::Map(map) => map.get(&name).cloned(),
                Value::MapRef(map) => map.borrow().get(&name).cloned(),
                v => return Err(Error::new(type_error!("map", v.type_of()), map_span)),
            }
            .ok_or_else(|| Error::new(format!("No property '{}'.", name), attr_span))
        }
        Reporter::Call(func, parameters) => {
            let func_span = func.span;
            let callable = match evaluate(scope, func.unbox())? {
                Value::Function(c) => c,
                v => return Err(Error::new(type_error!("function", v.type_of()), func_span)),
            };
            let parameters = parameters
                .into_iter()
                .map(|p| evaluate(scope, p))
                .collect::<Result<Vec<_>>>()?;
            callable.call(parameters, reporter.span)
        }
        Reporter::If { blocks, else_block } => {
            for (cond, body) in blocks {
                let cond_span = cond.span;
                if match evaluate(scope, cond)? {
                    Value::Boolean(b) => b,
                    v => return Err(Error::new(type_error!("boolean", v.type_of()), cond_span)),
                } {
                    return evaluate(scope, body);
                }
            }
            if let Some(body) = else_block {
                evaluate(scope, body.unbox())
            } else {
                Ok(Value::Null)
            }
        }
        Reporter::Add { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Int(int) => match evaluate(scope, b.unbox())? {
                    Value::Int(b) => Ok(Value::Int(int + b)),
                    Value::Float(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                Value::Float(float) => match evaluate(scope, b.unbox())? {
                    Value::Float(b) => Ok(Value::Float(float + b)),
                    Value::Int(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("number", v.type_of()), a_span)),
            }
        }
        Reporter::Subtract { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Int(int) => match evaluate(scope, b.unbox())? {
                    Value::Int(b) => Ok(Value::Int(int - b)),
                    Value::Float(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                Value::Float(float) => match evaluate(scope, b.unbox())? {
                    Value::Float(b) => Ok(Value::Float(float - b)),
                    Value::Int(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("number", v.type_of()), a_span)),
            }
        }
        Reporter::Multiply { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Int(int) => match evaluate(scope, b.unbox())? {
                    Value::Int(b) => Ok(Value::Int(int * b)),
                    Value::Float(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                Value::Float(float) => match evaluate(scope, b.unbox())? {
                    Value::Float(b) => Ok(Value::Float(float * b)),
                    Value::Int(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("number", v.type_of()), a_span)),
            }
        }
        Reporter::Divide { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Int(int) => match evaluate(scope, b.unbox())? {
                    Value::Int(b) => Ok(Value::Int(int / b)),
                    Value::Float(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                Value::Float(float) => match evaluate(scope, b.unbox())? {
                    Value::Float(b) => Ok(Value::Float(float / b)),
                    Value::Int(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("number", v.type_of()), a_span)),
            }
        }
        Reporter::Exponent { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Int(int) => match evaluate(scope, b.unbox())? {
                    Value::Int(b) => Ok(Value::Int(int.pow(b.try_into().unwrap_or(0)))),
                    Value::Float(_) => Err(Error::new(
                        "You cannot do arithmetic with floats and integers.",
                        b_span,
                    )),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                Value::Float(float) => match evaluate(scope, b.unbox())? {
                    Value::Float(b) => Ok(Value::Float(float.powf(b))),
                    Value::Int(b) => Ok(Value::Float(float.powi(b.try_into().unwrap_or(0)))),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("number", v.type_of()), a_span)),
            }
        }
        Reporter::Concat { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::String(a) => match evaluate(scope, b.unbox())? {
                    Value::String(b) => Ok(Value::String(a + &b)),
                    v => Err(Error::new(type_error!("string", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("string", v.type_of()), a_span)),
            }
        }
        Reporter::And { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Boolean(a) => match evaluate(scope, b.unbox())? {
                    Value::Boolean(b) => Ok(Value::Boolean(a && b)),
                    v => Err(Error::new(type_error!("boolean", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("boolean", v.type_of()), a_span)),
            }
        }
        Reporter::Or { a, b } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Boolean(a) => match evaluate(scope, b.unbox())? {
                    Value::Boolean(b) => Ok(Value::Boolean(a || b)),
                    v => Err(Error::new(type_error!("boolean", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("boolean", v.type_of()), a_span)),
            }
        }
        Reporter::Equality { a, b } => Ok(Value::Boolean(
            evaluate(scope, a.unbox())? == evaluate(scope, b.unbox())?,
        )),
        Reporter::Inequality { a, b, op } => {
            let (a_span, b_span) = (a.span, b.span);
            match evaluate(scope, a.unbox())? {
                Value::Int(a) => match evaluate(scope, b.unbox())? {
                    Value::Int(b) => Ok(Value::Boolean(if op.data == Comparison::LessThan {
                        a < b
                    } else {
                        a > b
                    })),
                    v => Err(Error::new(type_error!("integer", v.type_of()), b_span)),
                },
                Value::Float(a) => match evaluate(scope, b.unbox())? {
                    Value::Float(b) => Ok(Value::Boolean(if op.data == Comparison::LessThan {
                        a < b
                    } else {
                        a > b
                    })),
                    v => Err(Error::new(type_error!("float", v.type_of()), b_span)),
                },
                v => Err(Error::new(type_error!("number", v.type_of()), a_span)),
            }
        }
        Reporter::Not(value) => {
            let a_span = value.span;
            match evaluate(scope, value.unbox())? {
                Value::Boolean(a) => Ok(Value::Boolean(!a)),
                v => Err(Error::new(type_error!("boolean", v.type_of()), a_span)),
            }
        }
        Reporter::Negative(value) => {
            let a_span = value.span;
            match evaluate(scope, value.unbox())? {
                Value::Int(a) => Ok(Value::Int(-a)),
                Value::Float(a) => Ok(Value::Float(-a)),
                v => Err(Error::new(type_error!("boolean", v.type_of()), a_span)),
            }
        }
    }
}
