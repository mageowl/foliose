use crate::{error::Result, metakeys, span::Span};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    fmt::Debug,
    rc::Rc,
};

#[derive(Clone)]
pub enum Value {
    Null,
    String(String),
    Int(i32),
    Float(f64),
    Boolean(bool),

    Function(Rc<dyn Call>),
    Array(Vec<Self>),
    Map(HashMap<String, Self>),
    MapRef(Rc<RefCell<dyn MapRef>>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Int(arg0) => f.debug_tuple("Int").field(arg0).finish(),
            Self::Float(arg0) => f.debug_tuple("Float").field(arg0).finish(),
            Self::Boolean(arg0) => f.debug_tuple("Boolean").field(arg0).finish(),
            Self::Function(_) => f.debug_tuple("Function").finish(),
            Self::Array(arg0) => f.debug_tuple("Array").field(arg0).finish(),
            Self::Map(arg0) => f.debug_tuple("Map").field(arg0).finish(),
            Self::MapRef(_) => f.debug_tuple("MapRef").finish(),
        }
    }
}

impl Value {
    pub fn primative_type(&self) -> &str {
        match self {
            Value::Null => "null",
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Boolean(_) => "bool",
            Value::Function(_) => "function",
            Value::Array(_) => "array",
            Value::Map(_) => "map",
            Value::MapRef(_) => "map",
        }
    }
    pub fn type_of(&self) -> String {
        match self {
            Value::Map(map) => {
                if let Some(value) = map.get(metakeys::TYPE_NAME) {
                    match value {
                        Value::String(str) => str.clone(),
                        _ => String::from("map"),
                    }
                } else {
                    String::from("map")
                }
            }
            Value::MapRef(map) => {
                if let Some(value) = map.borrow().get(metakeys::TYPE_NAME) {
                    match value {
                        Value::String(str) => str.clone(),
                        _ => String::from("map"),
                    }
                } else {
                    String::from("map")
                }
            }
            _ => String::from(self.primative_type()),
        }
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Function(cb1), Value::Function(cb2)) => Rc::ptr_eq(cb1, cb2),
            (Value::Array(items1), Value::Array(items2)) => items1 == items2,
            (Value::Map(map1), Value::Map(map2)) => map1 == map2,
            (Value::MapRef(map1), Value::MapRef(map2)) => Rc::ptr_eq(map1, map2),
            _ => false,
        }
    }
}

pub trait Call {
    fn call(&self, args: Vec<Value>, span: Span) -> Result<Value>;
}
pub trait MapRef: Debug {
    fn get(&self, name: &str) -> Option<&Value>;
    fn set(&mut self, name: String, val: Value);
    fn parent(&self) -> Option<Rc<RefCell<dyn MapRef>>> {
        None
    }
    fn as_hashmap(&self) -> Option<&HashMap<String, Value>> {
        None
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}
impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Null
    }
}
impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(vec: Vec<T>) -> Self {
        Self::Array(vec.into_iter().map(Into::into).collect())
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        match value {
            Value::String(string) => string,
            _ => panic!("couldn't convert to string"),
        }
    }
}
impl From<Value> for i32 {
    fn from(value: Value) -> Self {
        match value {
            Value::Int(int) => int,
            _ => panic!("couldn't convert to string"),
        }
    }
}
impl From<Value> for Rc<RefCell<dyn MapRef>> {
    fn from(value: Value) -> Self {
        match value {
            Value::MapRef(map) => map,
            _ => panic!("couldn't convert to map ref"),
        }
    }
}
