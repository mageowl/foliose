use interpreter::{Scope, run_block};
use lib::error::Result;
use std::{cell::RefCell, env::args, fs, rc::Rc};
use std_lib::prelude::Prelude;

use compiler::{CompileChunk, CompilerScope};
use parser::Parse;

mod compiler;
mod interpreter;
mod lexer;
mod parser;
mod std_lib;

fn run(code: &str, prelude: Rc<RefCell<Prelude>>) -> Result<()> {
    let mut tokens = lexer::TokenStream::from(code);

    let program = parser::block::Block::parse(&mut tokens)?;
    //dbg!(&program);

    let mut prelude_scope = CompilerScope::from_prelude();
    let instruction_set = program.compile(&mut prelude_scope)?;
    //dbg!(&instruction_set);

    let scope = Rc::new(RefCell::new(Scope::new(Some(prelude))));
    run_block(&scope, instruction_set.data)?;
    Ok(())
}

fn main() {
    let mut args = args();
    args.next();
    let file = fs::read_to_string(args.next().unwrap()).unwrap();
    let prelude = Rc::new(RefCell::new(Prelude::new()));
    if let Err(e) = run(&file, prelude) {
        e.display(&file);
    }
}
