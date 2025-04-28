use interpreter::{Scope, run_block};
use lib::{error::Result, value::Value};
use std::{cell::RefCell, env::args, fs, rc::Rc};
use std_lib::{PRELUDE, init_registry};

use compiler::{CompileChunk, CompilerScope};
use parser::Parse;

mod compiler;
mod interpreter;
mod lexer;
mod parser;
mod std_lib;

pub fn run(code: &str) -> Result<Value> {
    let mut tokens = lexer::TokenStream::from(code);

    let program = parser::block::Block::parse(&mut tokens)?;
    //dbg!(&program);

    let mut prelude_scope = CompilerScope::from_prelude();
    let instruction_set = program.compile(&mut prelude_scope)?;
    //dbg!(&instruction_set);

    let scope = Rc::new(RefCell::new(Scope::new(Some(PRELUDE.with(Clone::clone)))));
    run_block(&scope, instruction_set.data)
}

fn main() {
    init_registry();

    let mut args = args();
    args.next();
    let file = fs::read_to_string(args.next().unwrap()).unwrap();
    if let Err(e) = run(&file) {
        e.display(&file);
    }
}
