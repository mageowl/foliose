use std::{cell::Cell, rc::Rc};

use lib::{
    compat::{function::IntoCallable, type_name::TypeName},
    error::*,
    interface,
    value::{Call, Value},
};

interface!(IterLib {
    range: range.into_callable(),
});

pub struct Range {
    idx: Cell<i32>,
    end: i32,
}
impl Call for Range {
    fn call(&self, _: Vec<Value>, _: lib::span::Span) -> Result<Value> {
        let current = self.idx.get();
        Ok(if current >= self.end {
            Value::Null
        } else {
            self.idx.replace(current + 1);
            Value::Int(current)
        })
    }
}
impl Into<Value> for Range {
    fn into(self) -> Value {
        Value::Function(Rc::new(self))
    }
}
impl TypeName for Range {
    fn type_name() -> String {
        String::from("std.iter.range")
    }
}

pub fn range(start: i32, end: i32) -> Result<Range> {
    Ok(Range {
        idx: Cell::new(start),
        end,
    })
}
