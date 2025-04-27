use lib::{
    error::{Error, Result},
    span::{Chunk, Span},
    token::Token,
};

use crate::{lexer::TokenStream, parser::parse_token};

use super::{Parse, block::Block, expression::Expression};

#[derive(Debug)]
pub enum Statement<'a> {
    Assign {
        name: Chunk<Expression<'a>>,
        op: Chunk<AssignOperator>,
        value: Chunk<Expression<'a>>,
    },
    Expr(Expression<'a>),

    While {
        cond: Chunk<Expression<'a>>,
        body: Chunk<Block<'a>>,
    },
    For {
        name: Chunk<&'a str>,
        iter: Chunk<Expression<'a>>,
        body: Chunk<Block<'a>>,
    },
    Return(Chunk<Expression<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignOperator {
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl<'a> Statement<'a> {
    fn parse_for(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        let start = parse_token(source, Token::KeywordFor)?.start;
        let name = <&str>::parse(source)?;
        parse_token(source, Token::KeywordIn)?;
        let iter = Expression::parse(source)?;
        parse_token(source, Token::BraceOpen)?;
        let body = Block::parse(source)?;
        let end = parse_token(source, Token::BraceClose)?.end;
        Ok(Chunk::new(
            Self::For { name, iter, body },
            Span { start, end },
        ))
    }
}

impl<'a> Parse<'a> for Statement<'a> {
    fn parse(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        match source.peek_token() {
            Some(Token::KeywordFor) => Self::parse_for(source),
            Some(Token::KeywordReturn) => {
                let Some(Ok(Chunk {
                    span: Span { start, .. },
                    ..
                })) = source.next()
                else {
                    unreachable!()
                };
                let expr = Expression::parse(source)?;
                let end = parse_token(source, Token::Semicolon)?.end;
                Ok(Chunk::new(Self::Return(expr), Span { start, end }))
            }
            Some(_) => {
                let expr = Expression::parse(source)?;
                if let Some(
                    Token::Equals
                    | Token::PlusEquals
                    | Token::MinusEquals
                    | Token::AsteriskEquals
                    | Token::SlashEquals,
                ) = source.peek_token()
                {
                    let Some(Ok(Chunk {
                        data: op_token,
                        span: op_span,
                    })) = source.next()
                    else {
                        unreachable!()
                    };
                    let value = Expression::parse(source)?;
                    let span = Span {
                        start: expr.span.start,
                        end: value.span.end,
                    };
                    parse_token(source, Token::Semicolon)?;
                    Ok(Chunk::new(
                        Self::Assign {
                            name: expr,
                            op: Chunk::new(
                                match op_token {
                                    Token::Equals => AssignOperator::Set,
                                    Token::PlusEquals => AssignOperator::Add,
                                    Token::MinusEquals => AssignOperator::Subtract,
                                    Token::AsteriskEquals => AssignOperator::Multiply,
                                    Token::SlashEquals => AssignOperator::Divide,
                                    _ => unreachable!(),
                                },
                                op_span,
                            ),
                            value,
                        },
                        span,
                    ))
                } else {
                    match &expr.data {
                        Expression::If { .. } | Expression::Block(_) => (),
                        _ => {
                            parse_token(source, Token::Semicolon)?;
                        }
                    }
                    Ok(expr.map(|expr| Self::Expr(expr)))
                }
            }
            None => Err(Error::new(
                "Expected a statement.",
                Span::char(*source.pos()),
            )),
        }
    }
}
