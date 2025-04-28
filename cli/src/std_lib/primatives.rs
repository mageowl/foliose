use std::{cell::RefCell, rc::Rc};

use lib::{
    compat::function::IntoCallable,
    error::*,
    interface,
    value::{MapRef, Value},
};

interface!(IntLib {
    to_str: int_to_str.into_callable()
});

fn int_to_str(int: i32) -> Result<String> {
    Ok(int.to_string())
}

interface!(Map {
    keys: map_keys.into_callable(),
    has: map_has.into_callable(),
});

fn map_keys(map: Rc<RefCell<dyn MapRef>>) -> Result<Vec<String>> {
    Ok(map
        .borrow()
        .as_hashmap()
        .map_or_else(Vec::new, |m| m.keys().cloned().collect()))
}

fn map_has(map: Rc<RefCell<dyn MapRef>>, name: String) -> Result<bool> {
    Ok(map.borrow().get(&name).is_some())
}
