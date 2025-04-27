use lib::{
    error::Result,
    span::{Chunk, Pos, Span},
    token::Token,
};

use crate::lexer::TokenStream;

use super::{Parse, statement::Statement};

#[derive(Debug)]
pub struct Block<'a> {
    pub body: Vec<Chunk<Statement<'a>>>,
}

impl<'a> Parse<'a> for Block<'a> {
    fn parse(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        let mut body = Vec::new();
        while source.peek().is_some() && source.peek_token() != Some(&Token::BraceClose) {
            body.push(Statement::parse(source)?);
        }
        let span = Span {
            start: body.get(0).map_or(Pos::default(), |c| c.span.start),
            end: body.last().map_or(Pos::default(), |c| c.span.start),
        };
        Ok(Chunk::new(Self { body }, span))
    }
}
