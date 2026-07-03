//! Parsing engine for building intermediate representations.

use std::{iter::Peekable, slice::Iter};

use crate::{
    ast::{Block, Stmt},
    errors::ParseError,
    expr::parse_expr,
    lexer::{Symbol, Token},
};

/// Used to share the parser's current state across functions.
struct ParseContext<'a> {
    /// A peekable iterator over slices of tokens.
    iter: Peekable<Iter<'a, &'a [Token]>>,
    /// The index (1-based) of the current line being parsed.
    line: u32,
}

/// Control flow indicators.
enum BlockTerminal {
    /// Beginning of an else branch.
    Else,
    /// End of the current scope.
    End,
    /// End of the token stream.
    EOF,
}

/// Parses a variable declaration from a slice of tokens.
///
/// A valid declaration statement contains the `var` keyword, followed by variable identifiers
/// that are delimited by commas. A variable declaration has two forms:
///
/// - *Single-Variable Declaration:* `var x`
/// - *Multi-Variable Declaration:* `var x, y, x`
///
/// The `tokens` argument is the slice of tokens following the `var` keyword.
fn parse_delcare(ctx: &ParseContext, tokens: &[Token]) -> Result<Stmt, ParseError> {
    let mut iter = tokens.iter();
    let mut vars = Vec::new();

    // extract first variable
    match iter.next() {
        Some(Token::Identifier(x)) => vars.push(x.clone()),
        _ => return Err(ParseError::InvalidDeclare(ctx.line)),
    }

    // parse remaining variables if they exist; check comma delimiter
    while let Some(tok) = iter.next() {
        match tok {
            Token::Comma => match iter.next() {
                Some(Token::Identifier(x)) => vars.push(x.clone()),
                None => return Err(ParseError::TrailingComma(ctx.line)),
                _ => return Err(ParseError::InvalidDeclare(ctx.line)),
            },
            _ => return Err(ParseError::MissingComma(ctx.line)),
        }
    }

    Ok(Stmt::Declare(vars))
}

/// Parses an assumption from a slice of tokens.
///
/// A valid assume statement contains the `assume` keyword followed by an expression.
///
/// ```txt
/// assume 1 + 0 < 2
/// ```
///
/// The `tokens` argument is the slice of tokens following the `assume` keyword.
fn parse_assume(ctx: &ParseContext, tokens: &[Token]) -> Result<Stmt, ParseError> {
    Ok(Stmt::Assume(parse_expr(tokens, ctx.line)?))
}

/// Parses an assertion from a slice of tokens.
///
/// A valid assert statement contains the `assert` keyword followed by an expression.
///
/// ```text
/// assert x != 0
/// ```
///
/// The `tokens` argument is the slice of tokens following the `assert` keyword.
fn parse_assert(ctx: &ParseContext, tokens: &[Token]) -> Result<Stmt, ParseError> {
    Ok(Stmt::Assert(parse_expr(tokens, ctx.line)?))
}

/// Parses a variable assignment from a slice of tokens.
///
/// A valid assign statement contains a variable identifier, followed by the assignment symbol and
/// an expression.
///
/// ```txt
/// x := 4 + 3
/// ```
///
/// The `head` argument is a reference to the variable identifier token and the `tokens` argument
/// is the slice of tokens after the assignment symbol.
fn parse_assign(ctx: &ParseContext, head: &Token, tokens: &[Token]) -> Result<Stmt, ParseError> {
    match head {
        Token::Identifier(x) => Ok(Stmt::Assign(x.clone(), parse_expr(tokens, ctx.line)?)),
        _ => Err(ParseError::InvalidAssign(ctx.line)),
    }
}

/// Parses an if-then-else conditional from a slice of tokens.
///
/// A valid if statement contains a consequent branch and an optional alternative branch. These two
/// branches are the "then" and "else" branches.
///
/// ```txt
/// if x then
///     x := true
/// else
///     x := false
/// end
/// ```
///
/// The `tokens` argument is the slice of tokens between the `if` and `then` keywords; it is parsed
/// as the conditional expression.
///
/// The `ctx` argument is the parsing context. The context's iterator is used to read succeeding
/// token slices, which contain statements for both branches and the `else`/`end` keywords.
fn parse_if(ctx: &mut ParseContext, tokens: &[Token]) -> Result<Stmt, ParseError> {
    let cond = parse_expr(tokens, ctx.line)?;
    let (then_branch, terminal) = parse_block(ctx)?;

    let else_branch = match terminal {
        BlockTerminal::End => None,
        BlockTerminal::Else => match parse_block(ctx)? {
            (else_block, BlockTerminal::End) => Some(else_block),
            _ => return Err(ParseError::ExpectedEnd(ctx.line)),
        },
        _ => return Err(ParseError::ExpectedElseEnd(ctx.line)),
    };

    Ok(Stmt::If {
        condition: cond,
        then_branch: then_branch,
        else_branch: else_branch,
    })
}

/// Parses a while loop from a slice of tokens.
///
/// A valid while loop statement contains a loop condition and an invariant. The loop's body
/// contains a sequence of statements that are repeatedly executed as long as the condition
/// and invariant hold.
///
/// ```txt
/// while x > 0
/// invariant x != 0
///     x := x - 1
/// end
/// ```
///
/// The `tokens` argument is the slice of tokens following the `while` keyword; it is parsed as the
/// loop condition.
///
/// The `ctx` argument is the parsing context. The context's iterator is used to read succeding
/// slices, which contain the invariant, loop body, and `else`/`end` keywords.
fn parse_while(ctx: &mut ParseContext, tokens: &[Token]) -> Result<Stmt, ParseError> {
    let cond = parse_expr(tokens, ctx.line)?;
    let inv = match {
        ctx.line += 1;
        ctx.iter.next()
    } {
        Some([Token::Invariant, inv_tokens @ ..]) => parse_expr(inv_tokens, ctx.line)?,
        _ => return Err(ParseError::ExpectedInvariant(ctx.line)),
    };

    let body = match parse_block(ctx)? {
        (block, BlockTerminal::End) => block,
        _ => return Err(ParseError::ExpectedEnd(ctx.line)),
    };

    Ok(Stmt::While {
        condition: cond,
        invariant: inv,
        body: body,
    })
}

/// Parses a statement from a slice of tokens.
fn parse_statement(ctx: &mut ParseContext, tokens: &[Token]) -> Result<Stmt, ParseError> {
    match tokens {
        [Token::Declare, var_toks @ ..] => parse_delcare(ctx, var_toks),
        [Token::Assume, expr_toks @ ..] => parse_assume(ctx, expr_toks),
        [Token::Assert, expr_toks @ ..] => parse_assert(ctx, expr_toks),
        [x, Token::Op(Symbol::Assign), expr_toks @ ..] => parse_assign(ctx, x, expr_toks),
        [Token::If, expr_toks @ .., Token::Then] => parse_if(ctx, expr_toks),
        [Token::While, expr_toks @ ..] => parse_while(ctx, expr_toks),
        [Token::Else] => Ok(Stmt::Else),
        [Token::End] => Ok(Stmt::End),
        [] => Ok(Stmt::Empty),

        _ => Err(ParseError::InvalidStatement(ctx.line)),
    }
}

/// Parses a sequence of statements.
fn parse_block(ctx: &mut ParseContext) -> Result<(Block, BlockTerminal), ParseError> {
    let mut block = Vec::new();

    while let Some(tokens) = {
        ctx.line += 1;
        ctx.iter.next()
    } {
        match parse_statement(ctx, tokens)? {
            Stmt::Empty => continue,
            Stmt::End => return Ok((Block(block), BlockTerminal::End)),
            Stmt::Else => return Ok((Block(block), BlockTerminal::Else)),
            stmt => block.push(stmt),
        }
    }

    Ok((Block(block), BlockTerminal::EOF))
}

/// Parses an abstract syntax tree (AST) from a token stream.
pub fn parse(tokens: &[Token]) -> Result<Block, ParseError> {
    let lines: Vec<&[Token]> = tokens.split(|tok| matches!(tok, Token::Newline)).collect();

    let mut ctx = ParseContext {
        iter: lines.iter().peekable(),
        line: 0,
    };

    match parse_block(&mut ctx)? {
        (ast, BlockTerminal::EOF) => Ok(ast),
        _ => Err(ParseError::ExpectedEof(ctx.line)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ast::Expr, errors::ParseError};

    #[test]
    fn test_declare_var() {
        assert_eq!(
            parse(&[Token::Declare, Token::Identifier("x".to_string())]).unwrap(),
            Block(vec![Stmt::Declare(vec!["x".to_string()])])
        );
    }

    #[test]
    fn test_declare_multivar() {
        assert_eq!(
            parse(&[
                Token::Declare,
                Token::Identifier("x".to_string()),
                Token::Comma,
                Token::Identifier("y".to_string())
            ])
            .unwrap(),
            Block(vec![Stmt::Declare(vec!["x".to_string(), "y".to_string()])])
        );
    }

    #[test]
    fn test_err_missing_comma() {
        assert_eq!(
            parse(&[
                Token::Declare,
                Token::Identifier("x".to_string()),
                Token::Identifier("y".to_string())
            ])
            .unwrap_err(),
            ParseError::MissingComma(1)
        );
    }

    #[test]
    fn test_err_trailing_comma() {
        assert_eq!(
            parse(&[
                Token::Declare,
                Token::Identifier("x".to_string()),
                Token::Comma,
                Token::Identifier("y".to_string()),
                Token::Comma,
            ])
            .unwrap_err(),
            ParseError::TrailingComma(1)
        );
    }

    #[test]
    fn test_assume() {
        assert_eq!(
            parse(&[Token::Assume, Token::Boolean(true)]).unwrap(),
            Block(vec![Stmt::Assume(Expr::Boolean(true))])
        );
    }

    #[test]
    fn test_assert() {
        assert_eq!(
            parse(&[Token::Assert, Token::Boolean(true)]).unwrap(),
            Block(vec![Stmt::Assert(Expr::Boolean(true))])
        );
    }

    #[test]
    fn test_assign() {
        assert_eq!(
            parse(&[
                Token::Identifier("x".to_string()),
                Token::Op(Symbol::Assign),
                Token::Integer(42)
            ])
            .unwrap(),
            Block(vec![Stmt::Assign("x".to_string(), Expr::Integer(42))])
        );
    }

    #[test]
    fn test_err_invalid_assign() {
        assert_eq!(
            parse(&[
                Token::Integer(42),
                Token::Op(Symbol::Assign),
                Token::Integer(42)
            ])
            .unwrap_err(),
            ParseError::InvalidAssign(1)
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(
            parse(&[
                Token::If,
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::Identifier("x".to_string()),
                Token::Op(Symbol::Assign),
                Token::Integer(1),
                Token::Newline,
                Token::End,
            ])
            .unwrap(),
            Block(vec![Stmt::If {
                condition: Expr::Boolean(true),
                then_branch: Block(vec![Stmt::Assign("x".to_string(), Expr::Integer(1))]),
                else_branch: None
            }])
        );
    }

    #[test]
    fn test_if_else() {
        assert_eq!(
            parse(&[
                Token::If,
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::Else,
                Token::Newline,
                Token::End,
            ])
            .unwrap(),
            Block(vec![Stmt::If {
                condition: Expr::Boolean(true),
                then_branch: Block(vec![]),
                else_branch: Some(Block(vec![]))
            }])
        );
    }

    #[test]
    fn test_if_nested() {
        assert_eq!(
            parse(&[
                Token::If,
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::If,
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::End,
                Token::Newline,
                Token::End,
            ])
            .unwrap(),
            Block(vec![Stmt::If {
                condition: Expr::Boolean(true),
                then_branch: Block(vec![Stmt::If {
                    condition: Expr::Boolean(true),
                    then_branch: Block(vec![]),
                    else_branch: None,
                }]),
                else_branch: None,
            }])
        );
    }

    #[test]
    fn test_err_expected_else_or_end() {
        assert_eq!(
            parse(&[Token::If, Token::Boolean(true), Token::Then]).unwrap_err(),
            ParseError::ExpectedElseEnd(2)
        );
    }

    #[test]
    fn test_err_unclosed_if() {
        assert_eq!(
            parse(&[
                Token::If,
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::Else,
                Token::Newline,
                Token::Identifier("x".to_string()),
                Token::Op(Symbol::Assign),
                Token::Integer(1),
            ])
            .unwrap_err(),
            ParseError::ExpectedEnd(4)
        );
    }

    #[test]
    fn test_while() {
        assert_eq!(
            parse(&[
                Token::While,
                Token::Identifier("x".to_string()),
                Token::Newline,
                Token::Invariant,
                Token::Boolean(true),
                Token::Newline,
                Token::Identifier("x".to_string()),
                Token::Op(Symbol::Assign),
                Token::Boolean(false),
                Token::Newline,
                Token::End,
            ])
            .unwrap(),
            Block(vec![Stmt::While {
                condition: Expr::Variable("x".to_string()),
                invariant: Expr::Boolean(true),
                body: Block(vec![Stmt::Assign("x".to_string(), Expr::Boolean(false))])
            }])
        );
    }

    #[test]
    fn test_err_missing_invariant() {
        assert_eq!(
            parse(&[
                Token::While,
                Token::Identifier("x".to_string()),
                Token::Newline,
                Token::Identifier("x".to_string()),
                Token::Op(Symbol::Assign),
                Token::Boolean(false),
                Token::Newline,
                Token::End,
            ])
            .unwrap_err(),
            ParseError::ExpectedInvariant(2)
        );
    }

    #[test]
    fn test_err_unclosed_while() {
        assert_eq!(
            parse(&[
                Token::While,
                Token::Identifier("x".to_string()),
                Token::Newline,
                Token::Invariant,
                Token::Boolean(true),
                Token::Newline,
                Token::Identifier("x".to_string()),
                Token::Op(Symbol::Assign),
                Token::Boolean(false),
                Token::Newline,
                // empty line
            ])
            .unwrap_err(),
            ParseError::ExpectedEnd(5)
        );
    }

    #[test]
    fn test_parser() {
        assert_eq!(
            parse(&[
                Token::Assume,
                Token::Boolean(true),
                Token::Newline,
                Token::Assert,
                Token::Boolean(true),
            ])
            .unwrap(),
            Block(vec![
                Stmt::Assume(Expr::Boolean(true)),
                Stmt::Assert(Expr::Boolean(true)),
            ])
        );
    }

    #[test]
    fn test_err_trailing_terminal() {
        assert_eq!(
            parse(&[Token::End]).unwrap_err(),
            ParseError::ExpectedEof(1)
        );
    }

    #[test]
    fn test_context() {
        let tokens = [
            Token::Identifier("x".to_string()),
            Token::Op(Symbol::Assign),
            Token::Integer(1),
            Token::Newline,
            // empty line
            Token::Newline,
            Token::End,
            Token::Newline,
            Token::Assert,
            Token::Boolean(true),
        ];

        let unparsed: Vec<&[Token]> = tokens.split(|tok| matches!(tok, Token::Newline)).collect();

        let mut ctx = ParseContext {
            iter: unparsed.iter().peekable(),
            line: 50,
        };

        assert!(parse_block(&mut ctx).is_ok());
        assert!(ctx.iter.peek().is_some());
        assert!(ctx.iter.next().unwrap() == &[Token::Assert, Token::Boolean(true)]);
        assert_eq!(ctx.line, 53);
    }
}
