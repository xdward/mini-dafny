//! Utility module for parsing expressions.

use std::{iter::Peekable, slice::Iter};

use crate::{
    ast::Expr,
    errors::ParseError,
    lexer::{Symbol, Token},
};

/// Used to share the expresion parser's current state across functions.
struct PrattParseContext<'tokens, 'src> {
    /// A peekable iterator over a slice of tokens.
    iter: Peekable<Iter<'tokens, Token<'src>>>,
    /// The index (1-based) of the current line being parsed.
    line: usize,
}

/// Returns the precedence level of an operator.
fn precedence(token: &Token, line: usize) -> Result<i64, ParseError> {
    match token {
        Token::Operator(sym) => match sym {
            Symbol::Mul => Ok(5),
            Symbol::Div => Ok(5),
            Symbol::Add => Ok(4),
            Symbol::Sub => Ok(4),
            Symbol::Lt => Ok(3),
            Symbol::Gt => Ok(3),
            Symbol::Le => Ok(3),
            Symbol::Ge => Ok(3),
            Symbol::Eq => Ok(2),
            Symbol::Neq => Ok(2),
            Symbol::And => Ok(1),
            Symbol::Or => Ok(1),
            _ => Err(ParseError::UnexpectedToken(line)),
        },
        _ => Err(ParseError::InvalidExpression(line)),
    }
}

/// Parses an atomic value.
fn parse_atom<'src>(ctx: &mut PrattParseContext<'_, 'src>) -> Result<Expr<'src>, ParseError> {
    match ctx.iter.next() {
        Some(Token::Operator(Symbol::LParen)) => {
            let expr = pratt_parser(ctx, 0)?;
            // peel right parenthesis
            match ctx.iter.next() {
                Some(Token::Operator(Symbol::RParen)) => Ok(expr),
                _ => Err(ParseError::UnclosedParenthesis(ctx.line)),
            }
        }
        Some(Token::Integer(i)) => Ok(Expr::Integer(*i)),
        Some(Token::Boolean(b)) => Ok(Expr::Boolean(*b)),
        Some(Token::Identifier(x)) => Ok(Expr::Variable(x)),
        _ => Err(ParseError::UnexpectedToken(ctx.line)),
    }
}

/// Parses expressions with operator precedence.
///
/// This implementation uses recursion and a peekable iterator to parse expressions by operator
/// level (see [precedence]). This implementation does not handle right-associative operators
/// such as *exponents* and *assignments*.
///
/// See: <https://en.wikipedia.org/wiki/Operator_associativity>
fn pratt_parser<'src>(
    ctx: &mut PrattParseContext<'_, 'src>,
    prev_prec: i64,
) -> Result<Expr<'src>, ParseError> {
    let mut lhs = parse_atom(ctx)?;

    while let Some(tok) = ctx.iter.peek() {
        if **tok == Token::Operator(Symbol::RParen) {
            break;
        }

        if !(precedence(tok, ctx.line)? > prev_prec) {
            break;
        }

        let op = ctx
            .iter
            .next()
            .ok_or(ParseError::InvalidExpression(ctx.line))?;
        let prec = precedence(op, ctx.line)?;
        let rhs = pratt_parser(ctx, prec)?;

        lhs = Expr::BinOp {
            lhs: Box::new(lhs),
            op: match op {
                Token::Operator(sym) => sym.clone(),
                _ => return Err(ParseError::InvalidExpression(ctx.line)),
            },
            rhs: Box::new(rhs),
        }
    }

    Ok(lhs)
}

/// Parses an expression from a slice of tokens.
pub fn parse_expr<'src>(tokens: &[Token<'src>], line: usize) -> Result<Expr<'src>, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError::MissingExpression(line));
    }

    let mut ctx = PrattParseContext {
        iter: tokens.iter().peekable(),
        line,
    };

    let expr = pratt_parser(&mut ctx, 0);

    if expr.is_ok() && ctx.iter.peek().is_some() {
        return Err(ParseError::TrailingParenthesis(line));
    }

    expr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParseError;

    #[test]
    fn test_operator_precedence() {
        let symbols = [
            Symbol::And,
            Symbol::Or,
            Symbol::Eq,
            Symbol::Neq,
            Symbol::Lt,
            Symbol::Gt,
            Symbol::Le,
            Symbol::Ge,
            Symbol::Add,
            Symbol::Sub,
            Symbol::Mul,
            Symbol::Div,
        ];
        let mut prev_prec = 0;
        for sym in symbols {
            let prec = precedence(&Token::Operator(sym), 0).unwrap();
            assert!(prec >= prev_prec);
            prev_prec = prec;
        }
    }

    #[test]
    fn test_literal_parsing() {
        let tokens = [Token::Integer(42)];
        assert_eq!(parse_expr(&tokens, 0).unwrap(), Expr::Integer(42));
    }

    #[test]
    fn test_expression_parsing() {
        let tokens = [
            Token::Integer(-10),
            Token::Operator(Symbol::Add),
            Token::Integer(4),
            Token::Operator(Symbol::Mul),
            Token::Integer(3),
            Token::Operator(Symbol::Eq),
            Token::Integer(2),
        ];

        assert_eq!(
            parse_expr(&tokens, 0).unwrap(),
            Expr::BinOp {
                lhs: Box::new(Expr::BinOp {
                    lhs: Box::new(Expr::Integer(-10)),
                    op: Symbol::Add,
                    rhs: Box::new(Expr::BinOp {
                        lhs: Box::new(Expr::Integer(4)),
                        op: Symbol::Mul,
                        rhs: Box::new(Expr::Integer(3)),
                    }),
                }),
                op: Symbol::Eq,
                rhs: Box::new(Expr::Integer(2)),
            }
        );
    }

    #[test]
    fn test_err_empty_expression() {
        assert_eq!(
            parse_expr(&[], 0).unwrap_err(),
            ParseError::MissingExpression(0)
        );
    }

    #[test]
    fn test_err_invalid_expression() {
        let tokens = [
            Token::Operator(Symbol::Add),
            Token::Identifier("x"),
            Token::Operator(Symbol::Add),
        ];
        assert_eq!(
            parse_expr(&tokens, 0).unwrap_err(),
            ParseError::UnexpectedToken(0)
        );
    }

    #[test]
    fn test_parenthesis() {
        let tokens = [
            Token::Operator(Symbol::LParen),
            Token::Integer(1),
            Token::Operator(Symbol::Add),
            Token::Integer(2),
            Token::Operator(Symbol::RParen),
            Token::Operator(Symbol::Add),
            Token::Integer(3),
        ];
        assert_eq!(
            parse_expr(&tokens, 0).unwrap(),
            Expr::BinOp {
                lhs: Box::new(Expr::BinOp {
                    lhs: Box::new(Expr::Integer(1)),
                    op: Symbol::Add,
                    rhs: Box::new(Expr::Integer(2)),
                }),
                op: Symbol::Add,
                rhs: Box::new(Expr::Integer(3)),
            }
        );
    }

    #[test]
    fn test_unclosed_parenthesis() {
        assert_eq!(
            parse_expr(&[Token::Operator(Symbol::LParen), Token::Integer(1)], 0).unwrap_err(),
            ParseError::UnclosedParenthesis(0)
        );
    }

    #[test]
    fn test_trailing_parenthesis() {
        assert_eq!(
            parse_expr(&[Token::Integer(1), Token::Operator(Symbol::RParen)], 0).unwrap_err(),
            ParseError::TrailingParenthesis(0)
        );
    }
}
