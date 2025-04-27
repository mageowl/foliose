use lib::{
    error::Result,
    instruction::Instruction,
    span::{Chunk, Span},
};

use crate::{parser::block::Block, std_lib::prelude::Prelude};

mod expression;
mod statement;

pub trait Compile<'a> {
    type Output;
    fn compile(self, span: Span, scope: &mut CompilerScope<'a, '_>) -> Result<Chunk<Self::Output>>;
}
pub trait CompileChunk<'a, T: Compile<'a>> {
    fn compile(self, scope: &mut CompilerScope<'a, '_>) -> Result<Chunk<T::Output>>;
}
impl<'a, T: Compile<'a>> CompileChunk<'a, T> for Chunk<T> {
    fn compile(
        self,
        scope: &mut CompilerScope<'a, '_>,
    ) -> Result<Chunk<<T as Compile<'a>>::Output>> {
        self.data.compile(self.span, scope)
    }
}

pub struct CompilerScope<'a, 'b> {
    variables: Vec<&'a str>,
    parent: Option<&'b Self>,
}
impl<'a, 'b> CompilerScope<'a, 'b> {
    pub fn new(parent: Option<&'b Self>) -> Self {
        Self {
            variables: Vec::new(),
            parent,
        }
    }

    fn get_var(&self, name: &'a str) -> Option<usize> {
        if self.variables.contains(&name) {
            Some(0)
        } else {
            self.parent.and_then(|p| p.get_var(name)).map(|i| i + 1)
        }
    }

    pub fn from_prelude() -> Self {
        Self {
            variables: Prelude::keys(),
            parent: None,
        }
    }
}

impl<'a> Compile<'a> for Block<'a> {
    type Output = Vec<Chunk<Instruction<'a>>>;
    fn compile(
        self,
        span: Span,
        scope: &mut CompilerScope<'a, '_>,
    ) -> Result<Chunk<Vec<Chunk<Instruction<'a>>>>> {
        let mut scope = CompilerScope::new(Some(scope));

        let mut instructions = Vec::new();
        for statement in self.body {
            instructions.push(statement.compile(&mut scope)?);
        }
        Ok(Chunk::new(instructions, span))
    }
}
