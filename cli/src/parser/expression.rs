use lib::{
    error::{Error, Result},
    span::{Chunk, Span},
    token::Token,
};

use crate::lexer::TokenStream;

use super::{Parse, block::Block, parse_token};

#[derive(Debug)]
pub enum Expression<'a> {
    Null,
    String(&'a str),
    Int(i32),
    Float(f64),
    Boolean(bool),
    Array(Vec<Chunk<Self>>),
    Function {
        parameters: Vec<Chunk<&'a str>>,
        body: Chunk<Box<Self>>,
    },

    Variable(&'a str),
    GetProp(Chunk<Box<Self>>, Chunk<&'a str>),
    DynProp(Chunk<Box<Self>>, Chunk<Box<Self>>),
    Call {
        value: Chunk<Box<Self>>,
        args: Vec<Chunk<Self>>,
    },

    Block(Block<'a>),
    If {
        blocks: Vec<(Chunk<Self>, Chunk<Self>)>,
        else_block: Option<Chunk<Box<Self>>>,
    },
    Import(Chunk<&'a str>),

    BinaryOp {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
        op: Chunk<Operation>,
    },
    UnaryOp {
        a: Chunk<Box<Self>>,
        op: Chunk<UnaryOperation>,
    },
    Group(Box<Self>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Operation {
    Concat,
    Exponent,
    Multiply,
    Divide,
    Add,
    Subtract,
    Equals,
    NotEquals,
    Lt,
    LtEqual,
    Gt,
    GtEqual,
    And,
    Or,
}
#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOperation {
    Not,
    Negative,
}

impl<'a> Expression<'a> {
    fn parse_single(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        let mut expr = match source.peek_token() {
            Some(Token::KeywordNull) => {
                let Some(Ok(Chunk { span, .. })) = source.next() else {
                    unreachable!()
                };
                Chunk::new(Self::Null, span)
            }
            Some(Token::String(_)) => {
                let Some(Ok(chunk)) = source.next() else {
                    unreachable!()
                };
                chunk.map(|data| {
                    let Token::String(str) = data else {
                        unreachable!()
                    };
                    Self::String(str)
                })
            }
            Some(Token::Int(_)) => {
                let Some(Ok(chunk)) = source.next() else {
                    unreachable!()
                };
                chunk.map(|data| {
                    let Token::Int(int) = data else {
                        unreachable!()
                    };
                    Self::Int(int)
                })
            }
            Some(Token::Float(_)) => {
                let Some(Ok(chunk)) = source.next() else {
                    unreachable!()
                };
                chunk.map(|data| {
                    let Token::Float(float) = data else {
                        unreachable!()
                    };
                    Self::Float(float)
                })
            }
            Some(Token::Boolean(_)) => {
                let Some(Ok(chunk)) = source.next() else {
                    unreachable!()
                };
                chunk.map(|data| {
                    let Token::Boolean(bool) = data else {
                        unreachable!()
                    };
                    Self::Boolean(bool)
                })
            }

            Some(Token::Ident(_)) => {
                let Some(Ok(chunk)) = source.next() else {
                    unreachable!()
                };
                chunk.map(|data| {
                    let Token::Ident(name) = data else {
                        unreachable!()
                    };
                    Self::Variable(name)
                })
            }

            Some(Token::ParenOpen) => {
                source.next();
                let expr = Self::parse(source)?;
                parse_token(source, Token::ParenClose)?;
                Chunk::new(Self::Group(Box::new(expr.data)), expr.span)
            }
            Some(Token::BraceOpen) => {
                let start = parse_token(source, Token::BraceOpen)?.start;
                let block = Block::parse(source)?.data;
                let end = parse_token(source, Token::BraceClose)?.end;
                Chunk::new(Self::Block(block), Span { start, end })
            }
            Some(Token::BracketOpen) => {
                let start = parse_token(source, Token::BracketOpen)?.start;
                let items = Self::parse_group(source, Token::Comma, Token::BracketClose)?.0;
                Chunk::new(
                    Self::Array(items),
                    Span {
                        start,
                        end: *source.pos(),
                    },
                )
            }

            Some(Token::KeywordFn) => Self::parse_fn(source)?,
            Some(Token::KeywordIf) => Self::parse_if(source)?,
            Some(Token::KeywordImport) => {
                let start = parse_token(source, Token::KeywordImport)?.start;
                parse_token(source, Token::ParenOpen)?;
                let path = match source.next().transpose()? {
                    Some(Chunk {
                        data: Token::String(path),
                        span,
                    }) => Chunk::new(path, span),
                    Some(ch) => {
                        return Err(Error::new(
                            "Expected a string literal. The import function must have a constant path.",
                            ch.span,
                        ));
                    }
                    None => {
                        return Err(Error::new(
                            "Expected a string literal.",
                            Span::char(*source.pos()),
                        ));
                    }
                };
                let end = parse_token(source, Token::ParenClose)?.end;
                Chunk::new(Self::Import(path), Span { start, end })
            }

            Some(Token::Bang | Token::Minus) => {
                let Some(Ok(Chunk { span, data: op })) = source.next() else {
                    unreachable!();
                };
                let a = Self::parse_single(source)?.as_box();
                let a_span = a.span;

                Chunk::new(
                    Self::UnaryOp {
                        a,
                        op: Chunk::new(
                            match op {
                                Token::Bang => UnaryOperation::Not,
                                Token::Minus => UnaryOperation::Negative,
                                _ => unreachable!(),
                            },
                            span,
                        ),
                    },
                    Span {
                        start: span.start,
                        end: a_span.end,
                    },
                )
            }
            Some(_) => {
                let Some(Ok(Chunk { span, data: token })) = source.next() else {
                    unreachable!()
                };
                return Err(Error::new(
                    format!("Expected an expression, but got {}.", token.name()),
                    span,
                ));
            }
            None => {
                source.next().transpose()?;
                return Err(Error::new(
                    "Expected an expression.",
                    Span::char(*source.pos()),
                ));
            }
        };

        while let Some(Token::Period | Token::BracketOpen | Token::ParenOpen) = source.peek_token()
        {
            let start = expr.span.start;
            match source.next().unwrap()?.data {
                Token::ParenOpen => {
                    expr = Chunk::new(
                        Self::Call {
                            value: expr.as_box(),
                            args: Self::parse_group(source, Token::Comma, Token::ParenClose)?.0,
                        },
                        Span {
                            start,
                            end: *source.pos(),
                        },
                    );
                }
                Token::Period => {
                    if let Some(Token::ParenOpen) = source.peek_token() {
                        source.next();
                        let property = Self::parse(source)?;
                        parse_token(source, Token::ParenClose)?;

                        expr = Chunk::new(
                            Self::DynProp(expr.as_box(), property.as_box()),
                            Span {
                                start,
                                end: *source.pos(),
                            },
                        )
                    } else {
                        let property = <&str>::parse(source)?;

                        expr = Chunk::new(
                            Self::GetProp(expr.as_box(), property),
                            Span {
                                start,
                                end: *source.pos(),
                            },
                        )
                    }
                }
                _ => (),
            }
        }

        Ok(expr)
    }

    fn parse_if(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        let start = *source.pos();
        let mut blocks = Vec::new();
        let mut else_block = None;

        while let Some(Token::KeywordIf) = source.peek_token() {
            source.next();
            let expr = Self::parse(source)?;
            let block_expr = matches!(source.peek_token(), Some(Token::BraceOpen));
            if !block_expr {
                parse_token(source, Token::Arrow)?;
            }
            blocks.push((expr, Self::parse(source)?));
            if !block_expr {
                parse_token(source, Token::Semicolon)?;
            }

            if let Some(Token::KeywordElse) = source.peek_token() {
                source.next();
                if let Some(Token::KeywordIf) = source.peek_token() {
                    continue;
                } else {
                    let block_expr = matches!(source.peek_token(), Some(Token::BraceOpen));
                    if !block_expr {
                        parse_token(source, Token::Arrow)?;
                    }
                    else_block = Some(Self::parse(source)?.as_box());
                    if !block_expr {
                        parse_token(source, Token::Semicolon)?;
                    }
                }
            }
        }

        Ok(Chunk::new(
            Self::If { blocks, else_block },
            Span {
                start,
                end: *source.pos(),
            },
        ))
    }

    fn parse_fn(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        let start = parse_token(source, Token::KeywordFn)?.start;
        parse_token(source, Token::ParenOpen)?;
        let parameters = <&str>::parse_group(source, Token::Comma, Token::ParenClose)?.0;
        parse_token(source, Token::Arrow)?;
        let body = Self::parse(source)?.as_box();
        let end = body.span.end;
        Ok(Chunk::new(
            Self::Function { parameters, body },
            Span { start, end },
        ))
    }
}

impl<'a> Parse<'a> for Expression<'a> {
    fn parse(source: &mut TokenStream<'a>) -> Result<Chunk<Self>> {
        let mut expr = Self::parse_single(source)?;

        fn insert<'a>(
            lhs: Chunk<Box<Expression<'a>>>,
            op: Chunk<Operation>,
            rhs: Chunk<Box<Expression<'a>>>,
        ) -> Chunk<Expression<'a>> {
            let span = Span {
                start: lhs.span.start,
                end: rhs.span.end,
            };

            match *lhs.data {
                Expression::BinaryOp { a, b, op: b_op } if op.data <= b_op.data => Chunk::new(
                    Expression::BinaryOp {
                        a,
                        b: insert(b, op, rhs).as_box(),
                        op: b_op,
                    },
                    span,
                ),
                _ => Chunk::new(Expression::BinaryOp { a: lhs, b: rhs, op }, span),
            }
        }

        while let Some(
            Token::Plus
            | Token::Minus
            | Token::Asterisk
            | Token::Slash
            | Token::DoubleEquals
            | Token::NotEquals
            | Token::Gt
            | Token::GtEquals
            | Token::Lt
            | Token::LtEquals
            | Token::DoubleAmpersand
            | Token::DoublePipe
            | Token::DoublePeriod,
        ) = source.peek_token()
        {
            let op = source.next().unwrap()?.map(|t| match t {
                Token::DoublePeriod => Operation::Concat,
                Token::Plus => Operation::Add,
                Token::Minus => Operation::Subtract,
                Token::Asterisk => Operation::Multiply,
                Token::Slash => Operation::Divide,
                Token::DoubleEquals => Operation::Equals,
                Token::NotEquals => Operation::NotEquals,
                Token::Gt => Operation::Gt,
                Token::GtEquals => Operation::GtEqual,
                Token::Lt => Operation::Lt,
                Token::LtEquals => Operation::LtEqual,
                Token::DoubleAmpersand => Operation::And,
                Token::DoublePipe => Operation::Or,
                _ => unreachable!(),
            });

            let b_expr = Self::parse_single(source)?;
            expr = insert(expr.as_box(), op, b_expr.as_box());
        }

        Ok(expr)
    }
}
