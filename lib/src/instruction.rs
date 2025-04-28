use crate::span::Chunk;

pub mod owned;

#[derive(Debug, Clone)]
pub enum Instruction<'a> {
    Set {
        map: Chunk<Reporter<'a>>,
        name: Chunk<&'a str>,
        value: Chunk<Reporter<'a>>,
    },
    While {
        condition: Chunk<Reporter<'a>>,
        body: Chunk<Vec<Chunk<Instruction<'a>>>>,
    },
    For {
        name: Chunk<&'a str>,
        iter: Chunk<Reporter<'a>>,
        body: Chunk<Vec<Chunk<Instruction<'a>>>>,
    },
    Return(Chunk<Reporter<'a>>),
    Void(Reporter<'a>),
}

#[derive(Debug, Clone)]
pub enum Reporter<'a> {
    /// 0 = current scope, 1 = parent, and so on
    Parent(usize),

    Null,
    ConstStr(&'a str),
    ConstInt(i32),
    ConstFloat(f64),
    ConstBool(bool),

    Block(Vec<Chunk<Instruction<'a>>>),
    Array(Chunk<Vec<Chunk<Self>>>),
    Function {
        parameters: Vec<Chunk<&'a str>>,
        body: Chunk<Box<Self>>,
    },

    Get {
        map: Chunk<Box<Self>>,
        name: Chunk<&'a str>,
    },
    DynGet {
        map: Chunk<Box<Self>>,
        attr: Chunk<Box<Self>>,
    },
    Call(Chunk<Box<Self>>, Vec<Chunk<Self>>),

    Import(Chunk<&'a str>),
    If {
        blocks: Vec<(Chunk<Self>, Chunk<Self>)>,
        else_block: Option<Chunk<Box<Self>>>,
    },

    Add {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Subtract {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Multiply {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Divide {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Exponent {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Concat {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    And {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Or {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Equality {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
    },
    Inequality {
        a: Chunk<Box<Self>>,
        b: Chunk<Box<Self>>,
        op: Chunk<Comparison>,
    },
    Not(Chunk<Box<Self>>),
    Negative(Chunk<Box<Self>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    GreaterThan,
    LessThan,
}
