use lib::{compat::function::IntoCallable, interface};

use super::{
    io, iter,
    primatives::{IntLib, Map},
    types::{self, TypeLib},
};

interface!(Prelude {
    int: IntLib::new(),
    map: Map::new(),
    iter: iter::IterLib::new(),
    r#type: TypeLib::new(),

    println: io::println.into_callable(),
    print: io::print.into_callable(),
});
