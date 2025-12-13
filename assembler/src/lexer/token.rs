use std::ops::Range;

use crate::lexer::{
    directive::Directive, instruction::Instruction, named_literal::NamedLiteral, symbol::Symbol,
};

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
    Directive,
    Sym,
    Instruction,
    NamedLiteral,
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
    Directive(Directive),
    Sym(Symbol),
    Instruction(Instruction),
    NamedLiteral(NamedLiteral),
    NumberLiteral(u8),
}

impl TokenValue {
    pub fn expect_directive(&self) -> Directive {
        match self {
            TokenValue::Directive(directive) => *directive,
            _ => panic!("Expected Directive token value"),
        }
    }

    pub fn expect_sym(&self) -> &Symbol {
        match self {
            TokenValue::Sym(symbol) => symbol,
            _ => panic!("Expected"),
        }
    }

    pub fn expect_instruction(&self) -> Instruction {
        match self {
            TokenValue::Instruction(instruction) => *instruction,
            _ => panic!("Expected Instruction token value"),
        }
    }

    pub fn expect_named_literal(&self) -> NamedLiteral {
        match self {
            TokenValue::NamedLiteral(named_literal) => *named_literal,
            _ => panic!("Expected NamedLiteral token value"),
        }
    }

    pub fn expect_number_literal(&self) -> u8 {
        match self {
            TokenValue::NumberLiteral(num) => *num,
            _ => panic!("Expected NumberLiteral token value"),
        }
    }
}
