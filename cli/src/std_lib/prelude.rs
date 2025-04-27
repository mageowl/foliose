use lib::{compat::function::IntoCallable, interface};

use super::{
    io, iter,
    primatives::{Integer, Map},
    types::{self, TypeLib},
};

interface!(Prelude {
    int: Integer::new(),
    map: Map::new(),

    println: io::println.into_callable(),
    print: io::print.into_callable(),

    range: iter::range.into_callable(),

    ty: TypeLib::new(),
    type_of: types::type_of.into_callable(),
});
