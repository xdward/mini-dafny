//! Abstract syntax tree (AST).

use crate::lexer::Symbol;

/// Represents an expression.
#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'src> {
    Integer(i64),
    Boolean(bool),
    Variable(&'src str),
    BinOp {
        lhs: Box<Expr<'src>>,
        op: Symbol,
        rhs: Box<Expr<'src>>,
    },
}

/// Represents a statement.
#[derive(Debug, PartialEq, Eq)]
pub enum Stmt<'src> {
    Declare(Vec<&'src str>),
    Assign(&'src str, Expr<'src>),
    Assume(Expr<'src>),
    Assert(Expr<'src>),
    If {
        condition: Expr<'src>,
        then_branch: Block<'src>,
        else_branch: Option<Block<'src>>,
    },
    While {
        condition: Expr<'src>,
        invariant: Expr<'src>,
        body: Block<'src>,
    },

    // helper variants for parsing control flow
    Else,
    End,
    Empty,
}

/// Represents a sequence of statements.
#[derive(Debug, PartialEq, Eq)]
pub struct Block<'src>(pub Vec<Stmt<'src>>);
