use std::{cell::RefCell, rc::Rc};

use crate::value::{MapRef, Value};

pub trait TypeName {
    fn type_name() -> String;
}

macro_rules! impl_type_name {
    ($($type: ty = $name: literal),*) => {
        $(
            impl TypeName for $type {
                fn type_name() -> String {
                    String::from($name)
                }
            }
        )*
    };
}

impl_type_name!(
    Value = "*",

    String = "string",
    i32 = "int",
    f64 = "float",
    bool = "bool",
    () = "null",
    Rc<RefCell<dyn MapRef>> = "map"
);

impl<T: TypeName> TypeName for Vec<T> {
    fn type_name() -> String {
        format!("array<{}>", T::type_name())
    }
}
