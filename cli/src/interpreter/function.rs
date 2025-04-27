use std::{cell::RefCell, rc::Rc};

use lib::{
    error::Result,
    instruction::{Reporter, owned::OwnedReporter},
    span::{Chunk, Span},
    value::{Call, Value},
};

use super::{Scope, evaluate};

pub struct Function {
    parent: Rc<RefCell<Scope>>,
    parameters: Vec<String>,
    body: OwnedReporter,
}

impl Function {
    pub fn new(parent: Rc<RefCell<Scope>>, parameters: Vec<String>, body: Chunk<Reporter>) -> Self {
        Self {
            parent,
            parameters,
            body: OwnedReporter::new(body),
        }
    }
}

impl Call for Function {
    fn call(&self, args: Vec<Value>, _span: Span) -> Result<Value> {
        let mut scope = Scope::new(Some(self.parent.clone()));
        for (name, value) in self.parameters.iter().zip(args.into_iter()) {
            scope.variables.insert(name.to_string(), value);
        }
        evaluate(&Rc::new(RefCell::new(scope)), self.body.borrow().clone())
    }
}
