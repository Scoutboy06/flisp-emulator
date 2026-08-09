use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Token {
    pub kind: TokenKind,
    pub value: TokenValue,
    pub span: Range<usize>,
}

impl Token {
    pub fn eof(pos: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            value: TokenValue::Empty,
            span: pos..pos,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TokenKind {
    #[default]
    Invalid,
    Eof,
    Newline,
    Identifier,
    NumberLiteral,
    ImmediatePrefix,
    Colon,
    Comma,
    Comment,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TokenValue {
    #[default]
    Empty,
    Identifier(String),
    NumberLiteral(u8),
}

impl TokenValue {
    pub fn expect_identifier(&self) -> &str {
        match self {
            TokenValue::Identifier(identifier) => identifier,
            _ => panic!("Expected identifier token value"),
        }
    }

    pub fn expect_number_literal(&self) -> u8 {
        match self {
            TokenValue::NumberLiteral(num) => *num,
            _ => panic!("Expected NumberLiteral token value"),
        }
    }
}
