use std::{marker::PhantomData, rc::Rc};
use variadics_please::all_tuples;

use super::type_name::TypeName;
use crate::{
    error::{Error, Result},
    span::Span,
    value::{Call, Value},
};

pub struct RsFunction<Fn, Marker> {
    function: Fn,
    parameters: Vec<String>,
    phantom: PhantomData<Marker>,
}

pub trait IntoCallable<Marker>: Sized {
    fn into_callable(self) -> RsFunction<Self, Marker>;
}

macro_rules! impl_fn {
    ($(($generic: ident, $var: ident)),*) => {
        impl<
            T: Fn($($generic),*) -> Result<R>,
            R: TypeName + Into<Value>,
            $($generic: TypeName + From<Value>),*
        > Call for RsFunction<T, (R, $($generic),*)> {
            #[allow(unused)]
            fn call(&self, args: Vec<Value>, span: Span) -> Result<Value> {
                let mut iter = args.into_iter();
                $(
                    let $var: $generic = iter.next().ok_or_else(|| Error::new("This function expected more arguments.", span))?.into();
                )*
                (self.function)($($var),*).map(|r| r.into()).map_err(|e| e.with_span(span))
            }
        }

        impl<
            T: Fn($($generic),*) -> Result<R>,
            R: TypeName + Into<Value>,
            $($generic: TypeName + From<Value>),*
        > IntoCallable<(R, $($generic),*)> for T {
            fn into_callable(self) -> RsFunction<T, (R, $($generic),*)> {
                RsFunction {
                    function: self,
                    parameters: vec![$($generic::type_name()),*],
                    phantom: PhantomData,
                }
            }

        }

        impl<
            T: Fn($($generic),*) -> Result<R> + 'static,
            R: TypeName + Into<Value> + 'static,
            $($generic: TypeName + From<Value> + 'static),*
        > Into<Value> for RsFunction<T, (R, $($generic),*)> {
            fn into(self) -> Value {
                Value::Function(Rc::new(self))
            }
        }
    };
}

all_tuples!(impl_fn, 0, 16, A, a);
