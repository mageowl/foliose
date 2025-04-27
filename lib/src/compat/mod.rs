pub mod function;
pub mod type_name;

#[macro_export]
macro_rules! interface {
    ($name: ident {
        $($var: ident: $value: expr),*
        $(,)?
    }) => {
        #[derive(Debug)]
        pub struct $name {
            $($var: $crate::value::Value),*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $($var: std::convert::Into::into($value)),*
                }
            }
            pub fn keys() -> Vec<&'static str> {
                vec![$(stringify!($var)),*]
            }
        }

        impl std::convert::From<$name> for $crate::value::Value {
            fn from(value: $name) -> $crate::value::Value {
                $crate::value::Value::MapRef(std::rc::Rc::new(std::cell::RefCell::new(value)))
            }
        }
        impl $crate::value::MapRef for $name {
            fn get(&self, name: &str) -> Option<&$crate::value::Value> {
                match name {
                    $(stringify!($var) => Some(&self.$var),)*
                    _ => None,
                }
            }
            fn set(&mut self, _name: std::string::String, _value: $crate::value::Value) {}
        }
    };
}
