use lib::{
    compat::function::IntoCallable, error::*, interface, span::Span, type_error, value::Value,
};

interface!(TypeLib {
    type_of: type_of.into_callable(),
    assert: assert.into_callable(),
});

pub fn type_of(value: Value) -> Result<String> {
    Ok(value.type_of())
}
pub fn assert(value: Value, expected: String) -> Result<()> {
    let vtype = type_of(value)?;
    if vtype != expected {
        Err(Error::new(type_error!(expected, vtype), Span::default()))
    } else {
        Ok(())
    }
}
