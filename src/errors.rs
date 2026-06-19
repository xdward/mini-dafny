//! Custom error types.

use std::fmt;

/// Errors produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    /// Received an empty input string.
    MissingInput,
    /// Encountered an invalid substring.
    InvalidToken(String),
    /// Encountered an invalid symbol.
    InvalidSymbol(String),
    /// Failed to parse an integer.
    InvalidInt(String),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingInput => write!(f, "input is missing"),
            Self::InvalidToken(tok) => write!(f, "invalid token\n-> \"{}\"", tok),
            Self::InvalidSymbol(tok) => write!(f, "invalid symbol\n-> \"{}\"", tok),
            Self::InvalidInt(tok) => write!(f, "failed to parse integer\n-> \"{}\"", tok),
        }
    }
}

impl std::error::Error for LexError {}

/// Alias for labeling the `u32` line number field in [ParseError].
type LineNumber = u32;

/// Errors produced by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Encountered an unexpected token.
    UnexpectedToken(LineNumber),
    /// Failed to parse an invalid declaration.
    InvalidDeclare(LineNumber),
    /// Failed to parse a declaration because of a missing comma.
    MissingComma(LineNumber),
    /// Failed to parse a declaration because of a trailing comma.
    TrailingComma(LineNumber),
    /// Failed to parse an invalid assignment.
    InvalidAssign(LineNumber),
    /// Failed to parse an invalid expression.
    InvalidExpression(LineNumber),
    /// Expected a closing parenthesis.
    UnclosedParenthesis(LineNumber),
    /// Encountered a trailing parenthesis.
    TrailingParenthesis(LineNumber),
    /// Expected an `else` or `end` terminal.
    ExpectedElseEnd(LineNumber),
    /// Expected the `end` terminal.
    ExpectedEnd(LineNumber),
    /// Encountered a trailing terminal.
    ExpectedEof(LineNumber),
    /// Expected an invariant statement.
    ExpectedInvariant(LineNumber),
    /// Encountered an invalid statement.
    InvalidStatement(LineNumber),
    /// Expected an expression in the statement.
    MissingExpression(LineNumber),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken(line) => write!(f, "unexpected token\n-> Line {}", line),
            Self::MissingComma(line) => write!(f, "missing comma\n-> Line {}", line),
            Self::TrailingComma(line) => write!(f, "trailing comma\n-> Line {}", line),
            Self::InvalidAssign(line) => write!(f, "invalid assignment\n-> Line {}", line),
            Self::InvalidExpression(line) => write!(f, "invalid expression\n-> Line {}", line),
            Self::UnclosedParenthesis(line) => write!(f, "unclosed parenthesis\n-> Line {}", line),
            Self::TrailingParenthesis(line) => write!(f, "trailing parenthesis\n-> Line {}", line),
            Self::InvalidDeclare(line) => write!(f, "invalid declaration\n-> Line {}", line),
            Self::ExpectedElseEnd(line) => write!(f, "expected else or end\n-> Line {}", line),
            Self::ExpectedEnd(line) => write!(f, "expected end\n-> Line {}", line),
            Self::ExpectedEof(line) => write!(f, "trailing terminal\n-> Line {}", line),
            Self::ExpectedInvariant(line) => write!(f, "expected invariant\n-> Line {}", line),
            Self::InvalidStatement(line) => write!(f, "invalid statement\n-> Line {}", line),
            Self::MissingExpression(line) => write!(f, "missing expression\n-> Line {}", line),
        }
    }
}

impl std::error::Error for ParseError {}

/// Errors produced by the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Encountered an illegal variable declaration.
    InvalidDeclaration,
    /// Variable has already been declared.
    AlreadyDeclared,
    /// Encountered an undeclared variable.
    UnboundVariable,
    /// Failed to compile an invalid expression.
    BadExpression,
    /// Failed to compile an invalid statement.
    BadStatement,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidDeclaration => write!(f, "variables may only be declared once"),
            Self::AlreadyDeclared => write!(f, "a variable has been declared more than once"),
            Self::UnboundVariable => write!(f, "attempted to use an undeclared variable"),
            Self::BadExpression => write!(f, "failed to evaluate expression"),
            Self::BadStatement => write!(f, "failed to evaluate statement"),
        }
    }
}

impl std::error::Error for CompileError {}
