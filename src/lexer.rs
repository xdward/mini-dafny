//! Lexical analyzer for token generation.

use crate::errors::LexError;
use regex::regex;

/// Represents expression operators.
///
/// There are four types of operations in the language:
///
/// - *Arithmetic* operations (`+`, `-`, `*`, `/`)
/// - *Relational* operations (`<`, `>`, `<=`, `>=`, `==`, `!=`)
/// - *Logical* operations (`&&`, `||`)
/// - *Assignment* operations (`:=`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Division (`/`)
    Div,
    /// Less than (`<`)
    Lt,
    /// Greater than (`>`)
    Gt,
    /// Less than or equal to (`<=`)
    Le,
    /// Greater than or equal to (`>=`)
    Ge,
    /// Equal to (`==`)
    Eq,
    /// Not equal to (`!=`)
    Neq,
    /// And (`&&`)
    And,
    /// Or (`||`)
    Or,
    /// Assign (`:=`)
    Assign,
    /// Left parenthesis (`(`)
    LParen,
    /// Right parenthesis (`)`)
    RParen,
}

/// A lexical token produced by the tokenizer.
///
/// There are four type of tokens:
///
/// - *Literal* tokens for variables and values
/// - *Keyword* tokens for statements and control flow
/// - *Symbolic* tokens for operators (see [Symbol])
/// - *Syntactic* tokens (for syntax)
#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    // Literals
    Identifier(&'a str),
    Integer(i64),
    Boolean(bool),

    // Keywords
    Declare,
    Assume,
    Assert,
    If,
    Then,
    Else,
    While,
    Invariant,
    End,

    // Symbols
    Operator(Symbol),

    // Syntax
    Comma,
    Newline,
}

/// Tokenizes a string slice.
///
/// Regex pattern matching is used to sequentially scan the source and create a [Token] for each
/// match found. A vector of tokens is returned if the source is a valid program. Otherwise,
/// [LexError] is returned for invalid matches or unrecognizable sequences.
pub fn tokenize<'src>(source: &'src str) -> Result<Vec<Token<'src>>, LexError> {
    if source.is_empty() {
        return Err(LexError::MissingInput);
    }

    let mut tokens = Vec::new();
    let mut index = 0; // position of the untokenized slice
    let mut line = 1; // current line being tokenized

    while index < source.len() {
        let unread = &source[index..];

        // match whitespace patterns
        if let Some(m) = regex!(r"^(?:\n|[ \t]+)").find(unread) {
            if m.as_str() == "\n" {
                tokens.push(Token::Newline);
                line += 1;
            }
            index += m.end();
            continue;
        }

        // match keyword patterns
        if let Some(m) = regex!(r"^[A-Za-z][0-9A-Za-z_]*").find(unread) {
            let token = match m.as_str() {
                "TRUE" => Token::Boolean(true),
                "FALSE" => Token::Boolean(false),
                "var" => Token::Declare,
                "assume" => Token::Assume,
                "assert" => Token::Assert,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                "while" => Token::While,
                "invariant" => Token::Invariant,
                "end" => Token::End,
                _ => Token::Identifier(m.as_str()),
            };
            tokens.push(token);
            index += m.end();
            continue;
        }

        // match number patterns
        if let Some(m) = regex!(r"^-?\d+").find(unread) {
            let token = m
                .as_str()
                .parse::<i64>()
                .map(Token::Integer)
                .map_err(|_| LexError::InvalidInt(line, m.as_str().to_string()))?;
            tokens.push(token);
            index += m.end();
            continue;
        }

        // match symbol patterns
        if let Some(m) = regex!(r"^(?:,|:=|==|!=|<=|>=|&&|\|\||[+\-*/<>()])").find(unread) {
            let token = match m.as_str() {
                "," => Token::Comma,
                "+" => Token::Operator(Symbol::Add),
                "-" => Token::Operator(Symbol::Sub),
                "*" => Token::Operator(Symbol::Mul),
                "/" => Token::Operator(Symbol::Div),
                "<" => Token::Operator(Symbol::Lt),
                ">" => Token::Operator(Symbol::Gt),
                "<=" => Token::Operator(Symbol::Le),
                ">=" => Token::Operator(Symbol::Ge),
                "==" => Token::Operator(Symbol::Eq),
                "!=" => Token::Operator(Symbol::Neq),
                "&&" => Token::Operator(Symbol::And),
                "||" => Token::Operator(Symbol::Or),
                ":=" => Token::Operator(Symbol::Assign),
                "(" => Token::Operator(Symbol::LParen),
                ")" => Token::Operator(Symbol::RParen),
                _ => return Err(LexError::InvalidSymbol(line, m.as_str().to_string())),
            };
            tokens.push(token);
            index += m.end();
            continue;
        }

        return Err(LexError::InvalidToken(line, unread.to_string()));
    }

    // require at least one token other than a newline
    if tokens.iter().all(|token| matches!(token, Token::Newline)) {
        return Err(LexError::MissingInput);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(
            tokenize("\n \t \n 1").unwrap(),
            vec![Token::Newline, Token::Newline, Token::Integer(1)]
        );
    }

    #[test]
    fn test_keywords() {
        let keywords = [
            ("TRUE", Token::Boolean(true)),
            ("FALSE", Token::Boolean(false)),
            ("var", Token::Declare),
            ("assume", Token::Assume),
            ("assert", Token::Assert),
            ("if", Token::If),
            ("then", Token::Then),
            ("else", Token::Else),
            ("while", Token::While),
            ("invariant", Token::Invariant),
            ("end", Token::End),
        ];

        for (keyword, expected) in keywords {
            assert_eq!(tokenize(keyword).unwrap(), vec![expected]);
        }
    }

    #[test]
    fn test_identifiers() {
        for ch in ('a'..='z').chain('A'..='Z') {
            let input = ch.to_string();
            assert_eq!(tokenize(&input).unwrap(), vec![Token::Identifier(&input)]);
        }

        for input in ["name_1", "X9", "variable_name"] {
            assert_eq!(tokenize(input).unwrap(), vec![Token::Identifier(input)]);
        }
    }

    #[test]
    fn test_integers() {
        for n in -10..=10 {
            assert_eq!(tokenize(&n.to_string()).unwrap(), vec![Token::Integer(n)]);
        }
    }

    #[test]
    fn test_symbols() {
        let operators = [
            ("+", Symbol::Add),
            ("-", Symbol::Sub),
            ("*", Symbol::Mul),
            ("/", Symbol::Div),
            ("<", Symbol::Lt),
            (">", Symbol::Gt),
            ("<=", Symbol::Le),
            (">=", Symbol::Ge),
            ("==", Symbol::Eq),
            ("!=", Symbol::Neq),
            ("&&", Symbol::And),
            ("||", Symbol::Or),
            (":=", Symbol::Assign),
            ("(", Symbol::LParen),
            (")", Symbol::RParen),
        ];

        for (operator, expected) in operators {
            assert_eq!(tokenize(operator).unwrap(), vec![Token::Operator(expected)]);
        }

        assert_eq!(tokenize(",").unwrap(), vec![Token::Comma]);
    }

    #[test]
    fn test_missing_input() {
        assert_eq!(tokenize(""), Err(LexError::MissingInput));
        assert_eq!(tokenize(" \t"), Err(LexError::MissingInput));
        assert_eq!(tokenize("\n\n"), Err(LexError::MissingInput));
    }

    #[test]
    fn test_invalid_tokens() {
        let allowed = [
            ">", "<", ">=", "<=", "&&", "||", ":=", "+", "-", "*", "/", ",", "(", ")",
        ];

        let invalid_inputs: Vec<String> = (0..0x7F)
            .filter_map(std::char::from_u32)
            .filter(|c| !c.is_ascii_alphanumeric())
            .filter(|c| !c.is_whitespace())
            .filter(|c| !allowed.iter().any(|s| s.starts_with(*c)))
            .map(|c| c.to_string())
            .collect();

        for input in invalid_inputs {
            assert_eq!(
                tokenize(&input).unwrap_err(),
                LexError::InvalidToken(1, input)
            );
        }
    }

    #[test]
    fn test_invalid_integer() {
        assert_eq!(
            tokenize("9223372036854775808"),
            Err(LexError::InvalidInt(1, "9223372036854775808".to_string()))
        );
    }

    #[test]
    fn test_debug_line() {
        assert_eq!(
            tokenize("var x\n@"),
            Err(LexError::InvalidToken(2, "@".to_string()))
        );
    }

    #[test]
    fn test_example() {
        let source = r#"
        var x, y
        x := 42
        if x >= 10 && TRUE then
            assert x != 0
        end
        "#;

        assert_eq!(
            tokenize(source).unwrap(),
            vec![
                Token::Newline,
                Token::Declare,
                Token::Identifier("x"),
                Token::Comma,
                Token::Identifier("y"),
                Token::Newline,
                Token::Identifier("x"),
                Token::Operator(Symbol::Assign),
                Token::Integer(42),
                Token::Newline,
                Token::If,
                Token::Identifier("x"),
                Token::Operator(Symbol::Ge),
                Token::Integer(10),
                Token::Operator(Symbol::And),
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::Assert,
                Token::Identifier("x"),
                Token::Operator(Symbol::Neq),
                Token::Integer(0),
                Token::Newline,
                Token::End,
                Token::Newline,
            ]
        );
    }
}
