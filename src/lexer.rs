//! Lexical analyzer for token generation.

use crate::errors::LexError;
use regex::Regex;
use std::sync::LazyLock;

/// Regular expression for whitespace patterns.
static RE_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:\n|[ \t]+)").unwrap());
/// Regular expression for word patterns.
static RE_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z][0-9A-Za-z_]*").unwrap());
/// Regular expression for number patterns.
static RE_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?\d+").unwrap());
/// Regular expression for symbol patterns.
static RE_SYMBOL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:,|:=|==|!=|<=|>=|&&|\|\||[+\-*/<>()])").unwrap());

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

impl Symbol {
    /// Parses a symbol from its string representation.
    pub fn from_str(s: &str) -> Result<Self, LexError> {
        match s {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            "<" => Ok(Self::Lt),
            ">" => Ok(Self::Gt),
            "<=" => Ok(Self::Le),
            ">=" => Ok(Self::Ge),
            "==" => Ok(Self::Eq),
            "!=" => Ok(Self::Neq),
            "&&" => Ok(Self::And),
            "||" => Ok(Self::Or),
            ":=" => Ok(Self::Assign),
            "(" => Ok(Self::LParen),
            ")" => Ok(Self::RParen),
            _ => Err(LexError::InvalidSymbol(s.to_string())),
        }
    }
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
pub enum Token {
    // Literals
    Identifier(String),
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
    Op(Symbol),

    // Syntax
    Comma,
    Newline,
}

/// Tokenizes a string slice.
///
/// Regex pattern matching is used to sequentially scan the source and create a [Token] for each
/// match found. A vector of tokens is returned if the source is a valid program. Otherwise,
/// [LexError] is returned for invalid matches or unrecognizable sequences.
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    if source.len() == 0 {
        return Err(LexError::MissingInput);
    }

    let mut tokens = Vec::new();
    let mut index = 0; // index pointer for the untokenized slice

    while index < source.len() {
        let unread = &source[index..];

        if let Some(m) = RE_WHITESPACE.find(unread) {
            if m.as_str() == "\n" {
                tokens.push(Token::Newline);
            }
            index += m.end();
            continue;
        }

        if let Some(m) = RE_WORD.find(unread) {
            let word = m.as_str();
            let token = match word {
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
                _ => Token::Identifier(word.to_string()),
            };
            tokens.push(token);
            index += m.end();
            continue;
        }

        if let Some(m) = RE_NUMBER.find(unread) {
            let number = m.as_str();
            let token = number
                .parse::<i64>()
                .map(Token::Integer)
                .map_err(|_| LexError::InvalidInt(number.to_string()))?;
            tokens.push(token);
            index += m.end();
            continue;
        }

        if let Some(m) = RE_SYMBOL.find(unread) {
            let token = match m.as_str() {
                "," => Token::Comma,
                sym => Token::Op(Symbol::from_str(sym)?),
            };
            tokens.push(token);
            index += m.end();
            continue;
        }

        return Err(LexError::InvalidToken(unread.to_string()));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(
            tokenize("\n \t \n").unwrap(),
            vec![Token::Newline, Token::Newline]
        );
    }

    #[test]
    fn test_keywords() {
        let keywords = [
            ("TRUE", vec![Token::Boolean(true)]),
            ("FALSE", vec![Token::Boolean(false)]),
            ("var", vec![Token::Declare]),
            ("assume", vec![Token::Assume]),
            ("assert", vec![Token::Assert]),
            ("if", vec![Token::If]),
            ("then", vec![Token::Then]),
            ("else", vec![Token::Else]),
            ("while", vec![Token::While]),
            ("invariant", vec![Token::Invariant]),
            ("end", vec![Token::End]),
        ];

        for (kw, expected) in keywords {
            assert_eq!(tokenize(kw).unwrap(), expected);
        }
    }

    #[test]
    fn test_identifiers() {
        for ch in ('a'..='z').chain('A'..='Z') {
            let input = ch.to_string();
            assert_eq!(tokenize(&input).unwrap(), vec![Token::Identifier(input)]);
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
            ("+", vec![Token::Op(Symbol::Add)]),
            ("-", vec![Token::Op(Symbol::Sub)]),
            ("*", vec![Token::Op(Symbol::Mul)]),
            ("/", vec![Token::Op(Symbol::Div)]),
            ("<", vec![Token::Op(Symbol::Lt)]),
            (">", vec![Token::Op(Symbol::Gt)]),
            ("<=", vec![Token::Op(Symbol::Le)]),
            (">=", vec![Token::Op(Symbol::Ge)]),
            ("==", vec![Token::Op(Symbol::Eq)]),
            ("!=", vec![Token::Op(Symbol::Neq)]),
            ("&&", vec![Token::Op(Symbol::And)]),
            ("||", vec![Token::Op(Symbol::Or)]),
            (":=", vec![Token::Op(Symbol::Assign)]),
            ("(", vec![Token::Op(Symbol::LParen)]),
            (")", vec![Token::Op(Symbol::RParen)]),
            (",", vec![Token::Comma]),
        ];

        for (op, expected) in operators {
            assert_eq!(tokenize(op).unwrap(), expected);
        }
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

        for inp in invalid_inputs {
            assert_eq!(tokenize(&inp).unwrap_err(), LexError::InvalidToken(inp));
        }
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
                Token::Identifier("x".into()),
                Token::Comma,
                Token::Identifier("y".into()),
                Token::Newline,
                Token::Identifier("x".into()),
                Token::Op(Symbol::Assign),
                Token::Integer(42),
                Token::Newline,
                Token::If,
                Token::Identifier("x".into()),
                Token::Op(Symbol::Ge),
                Token::Integer(10),
                Token::Op(Symbol::And),
                Token::Boolean(true),
                Token::Then,
                Token::Newline,
                Token::Assert,
                Token::Identifier("x".into()),
                Token::Op(Symbol::Neq),
                Token::Integer(0),
                Token::Newline,
                Token::End,
                Token::Newline,
            ]
        );
    }
}
