#![allow(unused)]

use crate::lexer::{
    directive::Directive, instruction::Instruction, named_literal::NamedLiteral, symbol::Symbol,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: TokenValue,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub fn eof(pos: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            value: TokenValue::Empty,
            start: pos,
            end: pos,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Invalid,
    Directive,
    Sym,
    Instruction,
    NamedLiteral,
    NumberLiteral,
    ImmediatePrefix,
    Colon,
    Comment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Empty,
    Directive(Directive),
    Sym(Symbol),
    Instruction(Instruction),
    NamedLiteral(NamedLiteral),
    NumberLiteral(u8),
}
