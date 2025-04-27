use lib::{
    error::{Error, Result},
    span::{Chunk, Span},
    token::Token,
};

use crate::lexer::TokenStream;

pub mod block;
pub mod expression;
pub mod statement;

pub trait Parse<'a>: Sized {
    fn parse(source: &mut TokenStream<'a>) -> Result<Chunk<Self>>;
    fn parse_group(
        source: &mut TokenStream<'a>,
        delimiter: Token,
        until: Token,
    ) -> Result<(Vec<Chunk<Self>>, bool)> {
        let mut vec = Vec::new();
        let mut trailing_delimiter = true;
        loop {
            if source.peek_token() == Some(&until) {
                source.next();
                break;
            } else if trailing_delimiter {
                trailing_delimiter = false;
                vec.push(Self::parse(source)?);
            } else {
                return Err(match source.next() {
                    Some(Ok(token)) => {
                        Error::new(format!("Expected {}.", delimiter.name()), token.span)
                    }
                    Some(Err(e)) => e,
                    None => Error::new(
                        format!("Expected {}.", delimiter.name()),
                        Span::char(*source.pos()),
                    ),
                });
            }
            if source.peek_token() == Some(&delimiter) {
                trailing_delimiter = true;
                source.next();
            }
        }
        Ok((vec, trailing_delimiter))
    }
}

impl<'a> Parse<'a> for &'a str {
    fn parse(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        match source.next() {
            Some(Ok(Chunk {
                span,
                data: Token::Ident(str),
            })) => Ok(Chunk::new(str, span)),
            Some(Ok(Chunk { span, .. })) => Err(Error::new("Expected an identifier.", span)),
            Some(Err(e)) => Err(e),
            None => Err(Error::new(
                "Expected an identifier.",
                Span::char(*source.pos()),
            )),
        }
    }
}

pub fn parse_token(source: &mut TokenStream, token: Token) -> Result<Span> {
    let next = source.next().unwrap_or_else(|| {
        Err(Error::new(
            format!("Expected {}.", token.name()),
            Span::char(*source.pos()),
        ))
    })?;
    if next.data == token {
        Ok(next.span)
    } else {
        Err(Error::new(format!("Expected {}.", token.name()), next.span))
    }
}
