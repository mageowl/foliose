use lib::interface;
use primatives::Integer;

pub mod io;
pub mod iter;
pub mod prelude;
pub mod primatives;
pub mod types;

interface!(StdLibrary {
    int: Integer::new(),
});
