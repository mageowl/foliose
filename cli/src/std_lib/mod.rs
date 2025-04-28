use std::{cell::RefCell, rc::Rc};

use io::IoLib;
use lib::module_registry::REGISTRY;
use prelude::Prelude;

pub mod fs;
pub mod io;
pub mod iter;
pub mod prelude;
pub mod primatives;
pub mod types;

thread_local!(pub static PRELUDE: Rc<RefCell<Prelude>> = Rc::new(RefCell::new(Prelude::new())));
pub fn init_registry() {
    REGISTRY.with_borrow_mut(|reg| {
        PRELUDE.with(|prelude| {
            let prelude = prelude.borrow();
            reg.insert("std/iter", prelude.iter.clone());
            reg.insert("std/int", prelude.int.clone());
            reg.insert("std/map", prelude.map.clone());
            reg.insert("std/type", prelude.r#type.clone());
        });
        reg.insert("std/io", IoLib::new());
        //reg.insert("std/fs", FileLib::new());
    })
}
