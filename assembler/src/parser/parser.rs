#![allow(unused)]

use std::{collections::HashMap, ops::Range};

use crate::lexer::{
    Lexer, NamedLiteral,
    instruction::Instruction,
    token::{Token, TokenKind, TokenValue},
};

pub struct Parser<'a> {
    source: &'a str,
    lexer: Lexer<'a>,
    curr_tok: Token,
}

#[derive(Debug, Clone)]
pub struct ProgramAST {
    instructions: Vec<AsmInstruction>,
    symbols: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct AsmInstruction {
    opcode: u8,
    operands: Vec<Operand>,
}

#[derive(Debug, Clone)]
enum OperandForm {
    None,

    // Just one operand: `n`, `X`, label, etc.
    One(Atom),

    // Something like `n,X` or `label,Y`
    Two(Atom, Atom),

    // Immediate: `#5` or `#label`
    Imm1(Atom),

    // Immediate with two operands: `#5,X` or `#label,Y`
    Imm2(Atom, Atom),
}

#[derive(Debug, Clone)]
pub enum Atom {
    Number(u8),
    Reg(NamedLiteral),
    Symbol(String),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Imm(u8),           // #Data
    AbsAdr(u8),        // Adr
    RelAdr(u8),        // Adr
    N(u8),             // n
    Reg(NamedLiteral), // X, Y, SP, etc.
}

fn op0(opcode: u8) -> AsmInstruction {
    AsmInstruction {
        opcode,
        operands: Vec::new(),
    }
}

fn op1(opcode: u8, a: Operand) -> AsmInstruction {
    AsmInstruction {
        opcode,
        operands: vec![a],
    }
}

fn op2(opcode: u8, a: Operand, b: Operand) -> AsmInstruction {
    AsmInstruction {
        opcode,
        operands: vec![a, b],
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
    span: Range<usize>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            source,
            lexer: Lexer::new(source),
            curr_tok: Token::default(),
        }
    }

    fn advance(&mut self) {
        self.curr_tok = self.lexer.next_token();
        dbg!(&self.curr_tok);
    }

    fn curr(&self) -> &Token {
        &self.curr_tok
    }

    fn curr_span(&self) -> Range<usize> {
        self.curr_tok.start..self.curr_tok.end
    }

    pub fn parse(&mut self) -> Result<ProgramAST, ParseError> {
        // Initialize the first token
        self.advance();

        let mut instructions: Vec<AsmInstruction> = Vec::new();
        let mut symbols: HashMap<String, usize> = HashMap::new();

        use TokenKind as TK;
        use TokenValue as TV;
        while self.curr().kind != TK::Eof {
            match self.curr().kind {
                TK::Instruction => {
                    let ins = self.parse_instruction()?;
                    instructions.push(ins);
                }
                TK::Sym => todo!(),
                TK::Directive => todo!(),
                _ => todo!("{:?}", self.curr()),
            };

            println!("---");
        }

        Ok(ProgramAST {
            instructions,
            symbols,
        })
    }

    fn parse_instruction(&mut self) -> Result<AsmInstruction, ParseError> {
        let ins = self.curr().value.expect_instruction();
        self.advance(); // Consume instruction token
        let ops = self.parse_operands()?;

        use Instruction as I;
        use OperandForm as OF;
        match (ins, &ops) {
            (I::SUBA, OF::Imm1(Atom::Number(n))) => Ok(op1(0x94, Operand::Imm(*n))),
            (I::SUBA, OF::One(Atom::Number(n))) => Ok(op1(0xA4, Operand::AbsAdr(*n))),
            (I::SUBA, OF::Two(Atom::Number(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xB4, Operand::N(*n)))
            }
            (I::SUBA, OF::Two(Atom::Number(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xC4, Operand::N(*n)))
            }
            (I::SUBA, OF::Two(Atom::Number(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xD4, Operand::N(*n)))
            }
            _ => todo!("{:?} - {:?}", &ins, &ops),
        }
    }

    fn parse_operands(&mut self) -> Result<OperandForm, ParseError> {
        use TokenKind as TK;
        use TokenValue as TV;

        match self.curr().kind {
            TK::ImmediatePrefix => {
                self.advance();
                let op1 = self.parse_op()?;
                match self.curr().kind {
                    TK::Comma => {
                        self.advance();
                        let op2 = self.parse_op()?;
                        Ok(OperandForm::Imm2(op1, op2))
                    }
                    _ => Ok(OperandForm::Imm1(op1)),
                }
            }
            TK::NamedLiteral | TK::NumberLiteral | TK::Sym => {
                let op1 = self.parse_op()?;
                match self.curr().kind {
                    TK::Comma => {
                        self.advance();
                        let op2 = self.parse_op()?;
                        Ok(OperandForm::Two(op1, op2))
                    }
                    _ => Ok(OperandForm::One(op1)),
                }
            }
            _ => {
                dbg!(&self.curr());
                Ok(OperandForm::None)
            }
        }
    }

    /// op := NamedLiteral | NumberLiteral | Sym
    fn parse_op(&mut self) -> Result<Atom, ParseError> {
        let val = match self.curr().kind {
            TokenKind::NamedLiteral => {
                let name_lit = self.curr().value.expect_named_literal();
                Ok(Atom::Reg(name_lit))
            }
            TokenKind::NumberLiteral => {
                let num_lit = self.curr().value.expect_number_literal();
                Ok(Atom::Number(num_lit))
            }
            TokenKind::Sym => {
                let sym = self.curr().value.expect_sym();
                Ok(Atom::Symbol(sym.0.to_owned()))
            }
            _ => Err(ParseError {
                message: "Expected operand".to_string(),
                span: self.curr_span(),
            }),
        }?;

        self.advance();
        Ok(val)
    }
}
