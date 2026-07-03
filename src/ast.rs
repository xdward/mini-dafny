//! Abstract syntax tree (AST).

use crate::lexer::Symbol;

/// Represents an expression.
#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Boolean(bool),
    Integer(i64),
    Variable(String),
    BinOp {
        lhs: Box<Expr>,
        op: Symbol,
        rhs: Box<Expr>,
    },
}

/// Represents a statement.
#[derive(Debug, PartialEq, Eq)]
pub enum Stmt {
    Declare(Vec<String>),
    Assign(String, Expr),
    Assume(Expr),
    Assert(Expr),
    If {
        condition: Expr,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    While {
        condition: Expr,
        invariant: Expr,
        body: Block,
    },

    // helper variants for parsing control flow
    Else,
    End,
    Empty,
}

/// Represents a sequence of statements.
#[derive(Debug, PartialEq, Eq)]
pub struct Block(pub Vec<Stmt>);
