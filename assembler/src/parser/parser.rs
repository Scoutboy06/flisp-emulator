use std::ops::Range;

use ariadne::{Label, Report, ReportKind, Source};

use crate::lexer::{
    Lexer, NamedLiteral,
    directive::Directive,
    instruction::Instruction,
    token::{Token, TokenKind},
};

#[derive(Debug)]
pub struct ProgramAST {
    pub lines: Vec<AsmLine>,
}

#[derive(Debug)]
pub enum AsmLine {
    Instruction(AsmInstruction),
    Directive(AsmDirective),
    Symbol(AsmSymbol),
}

#[derive(Debug, Clone)]
pub struct AsmInstruction {
    pub span: Range<usize>,
    pub opcode: u8,
    pub operands: Vec<Operand>,
}

impl AsmInstruction {
    /// The size of an instruction in bytes (1 byte for opcode + operands)
    ///
    /// The maximum size is 3 bytes (1 byte opcode + 2 bytes operands)
    pub fn size(&self) -> u8 {
        1 + self.operands.len() as u8
    }
}

#[derive(Debug, Clone)]
pub struct AsmDirective {
    pub span: Range<usize>,
    pub name: Directive,
    pub args: Vec<Atom>,
}

#[derive(Debug)]
pub struct AsmSymbol {
    pub span: Range<usize>,
    pub name: String,
}

#[derive(Debug, Clone)]
enum OperandForm {
    None,

    /// Just one operand: `n`, `X`, label, etc.
    One(Atom),

    /// Something like `n,X` or `label,Y`
    Two(Atom, Atom),

    /// Immediate: `#5` or `#label`
    Imm1(Atom),
}

#[derive(Debug, Clone)]
pub enum Atom {
    NumOrSym(NumOrSym),
    Reg(NamedLiteral),
    None,
}

#[derive(Debug, Clone)]
pub enum NumOrSym {
    Num(u8),
    Sym(String),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Imm(NumOrSym),     // #Data
    AbsAdr(NumOrSym),  // Adr
    RelAdr(NumOrSym),  // Adr
    N(NumOrSym),       // n
    Reg(NamedLiteral), // X, Y, SP, etc.
}

fn op0(opcode: u8) -> (u8, Vec<Operand>) {
    (opcode, Vec::new())
}

fn op1(opcode: u8, a: Operand) -> (u8, Vec<Operand>) {
    (opcode, vec![a])
}

#[allow(dead_code)]
fn op2(opcode: u8, a: Operand, b: Operand) -> (u8, Vec<Operand>) {
    (opcode, vec![a, b])
}

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub span: Range<usize>,
}

impl ParseError {
    pub fn new(msg: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            msg: msg.into(),
            span,
        }
    }

    pub fn report_on(&self, file_name: &str, src: &str) {
        self.build_report(file_name)
            .eprint((file_name, Source::from(src)))
            .unwrap();
    }

    pub fn build_report<'a>(&'a self, file_name: &'a str) -> Report<'a, (&'a str, Range<usize>)> {
        Report::build(ReportKind::Error, (file_name, self.span.to_owned()))
            .with_message(&self.msg)
            .with_label(Label::new((file_name, self.span.to_owned())).with_message("here"))
            .finish()
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    curr_tok: Token,
    prev_tok: Token,
    source_name: Option<String>,
}

impl<'a> Parser<'a> {
    pub fn from_source(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            curr_tok: Token::default(),
            prev_tok: Token::default(),
            source_name: None,
        }
    }

    pub fn with_source_name(mut self, name: String) -> Self {
        self.source_name = Some(name);
        self
    }

    fn advance(&mut self) {
        self.prev_tok = std::mem::take(&mut self.curr_tok);
        self.curr_tok = self.lexer.next_token();
    }

    fn curr(&self) -> &Token {
        &self.curr_tok
    }

    fn prev(&self) -> &Token {
        &self.prev_tok
    }

    fn curr_span(&self) -> Range<usize> {
        self.curr_tok.span.to_owned()
    }

    fn err(&self, msg: String, span: Range<usize>) -> ParseError {
        ParseError { msg, span }
    }

    pub fn parse(&mut self) -> Result<ProgramAST, ParseError> {
        // Initialize the first token
        self.advance();

        let mut lines: Vec<AsmLine> = Vec::new();

        use TokenKind as TK;
        // use TokenValue as TV;
        while self.curr().kind != TK::Eof {
            match self.curr().kind {
                TK::Instruction => {
                    let ins = self.parse_instruction()?;
                    lines.push(AsmLine::Instruction(ins));
                }
                TK::Sym => {
                    let name = self.curr().value.expect_sym();
                    let span = self.curr().span.to_owned();
                    lines.push(AsmLine::Symbol(AsmSymbol {
                        span: span.clone(),
                        name: name.0.to_owned(),
                    }));

                    self.advance();
                    if self.curr().kind == TK::Colon {
                        self.advance();
                    }
                }
                TK::Directive => {
                    let dir = self.parse_directive()?;
                    lines.push(AsmLine::Directive(dir));
                }
                _ => todo!("{:?}", self.curr()),
            };
        }

        Ok(ProgramAST { lines })
    }

    fn parse_directive(&mut self) -> Result<AsmDirective, ParseError> {
        let start_pos = self.curr().span.start;
        match self.curr().value.expect_directive() {
            Directive::Org => {
                self.advance();
                let span = start_pos..self.curr().span.end;
                match self.curr().kind {
                    TokenKind::NumberLiteral | TokenKind::Sym => Ok(AsmDirective {
                        span,
                        name: Directive::Org,
                        args: vec![self.parse_atom().unwrap()],
                    }),
                    _ => Err(self.err("Expected number or symbol".into(), span)),
                }
            }
            Directive::Equ => {
                self.advance();
                if matches!(self.curr().kind, TokenKind::NumberLiteral | TokenKind::Sym) {
                    let span = start_pos..self.curr().span.end;
                    Ok(AsmDirective {
                        span,
                        name: Directive::Equ,
                        args: vec![self.parse_atom().unwrap()],
                    })
                } else {
                    Err(self.err(
                        "Expected number or symbol".into(),
                        self.curr().span.to_owned(),
                    ))
                }
            }
            Directive::Fcb => {
                self.advance();
                let mut args: Vec<Atom> = Vec::new();

                while let TokenKind::NumberLiteral | TokenKind::Sym = self.curr().kind {
                    args.push(self.parse_atom()?);

                    if self.curr().kind == TokenKind::Comma {
                        self.advance(); // Consume comma
                    } else {
                        break;
                    }
                }
                let end = self.prev().span.end;
                Ok(AsmDirective {
                    span: start_pos..end,
                    name: Directive::Fcb,
                    args,
                })
            }
            Directive::Fcs => todo!(),
            Directive::Rmb => todo!(),
        }
    }

    fn parse_instruction(&mut self) -> Result<AsmInstruction, ParseError> {
        let start = self.curr().span.start;
        let ins = self.curr().value.expect_instruction();
        self.advance(); // Consume instruction token
        let ops = self.parse_operands()?;
        let end = self.prev().span.end;

        use Instruction as I;
        use OperandForm as OF;
        let res = match (ins, ops) {
            (I::NOP, OF::None) => Ok(op0(0x00)),
            (I::ANDCC, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x01, Operand::Imm(n))),
            (I::ORCC, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x02, Operand::Imm(n))),
            (I::CLRA, OF::None) => Ok(op0(0x05)),
            (I::NEGA, OF::None) => Ok(op0(0x06)),
            (I::INCA, OF::None) => Ok(op0(0x07)),
            (I::DECA, OF::None) => Ok(op0(0x08)),
            (I::TSTA, OF::None) => Ok(op0(0x09)),
            (I::COMA, OF::None) => Ok(op0(0x0a)),
            (I::LSLA, OF::None) => Ok(op0(0x0b)),
            (I::LSRA, OF::None) => Ok(op0(0x0c)),
            (I::ROLA, OF::None) => Ok(op0(0x0d)),
            (I::RORA, OF::None) => Ok(op0(0x0e)),
            (I::ASRA, OF::None) => Ok(op0(0x0f)),
            (I::PSHA, OF::None) => Ok(op0(0x10)),
            (I::PSHX, OF::None) => Ok(op0(0x11)),
            (I::PSHY, OF::None) => Ok(op0(0x12)),
            (I::PSHC, OF::None) => Ok(op0(0x13)),
            (I::PULA, OF::None) => Ok(op0(0x14)),
            (I::PULX, OF::None) => Ok(op0(0x15)),
            (I::PULY, OF::None) => Ok(op0(0x16)),
            (I::PULC, OF::None) => Ok(op0(0x17)),
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::CC))) => {
                Ok(op0(0x18))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::CC), Atom::Reg(NamedLiteral::A))) => {
                Ok(op0(0x19))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::X), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x1a))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::Y), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x1b))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::X), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op0(0x1c))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::SP), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x1d))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::Y), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op0(0x1e))
            }
            (I::TFR, OF::Two(Atom::Reg(NamedLiteral::SP), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x1f))
            }
            (I::BSR, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x20, Operand::RelAdr(n))),
            (I::BRA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x21, Operand::RelAdr(n))),
            (I::BMI, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x22, Operand::RelAdr(n))),
            (I::BPL, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x23, Operand::RelAdr(n))),
            (I::BEQ, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x24, Operand::RelAdr(n))),
            (I::BNE, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x25, Operand::RelAdr(n))),
            (I::BVS, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x26, Operand::RelAdr(n))),
            (I::BVC, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x27, Operand::RelAdr(n))),
            (I::BCS, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x28, Operand::RelAdr(n))),
            (I::BCC, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x29, Operand::RelAdr(n))),
            (I::BHI, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x2a, Operand::RelAdr(n))),
            (I::BLS, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x2b, Operand::RelAdr(n))),
            (I::BGT, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x2c, Operand::RelAdr(n))),
            (I::BGE, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x2d, Operand::RelAdr(n))),
            (I::BLE, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x2e, Operand::RelAdr(n))),
            (I::BLT, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x2f, Operand::RelAdr(n))),
            (I::STX, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x30, Operand::AbsAdr(n))),
            (I::STY, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x31, Operand::AbsAdr(n))),
            (I::STSP, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x32, Operand::AbsAdr(n))),
            (I::JMP, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x33, Operand::AbsAdr(n))),
            (I::JSR, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x34, Operand::AbsAdr(n))),
            (I::CLR, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x35, Operand::AbsAdr(n))),
            (I::NEG, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x36, Operand::AbsAdr(n))),
            (I::INC, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x37, Operand::AbsAdr(n))),
            (I::DEC, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x38, Operand::AbsAdr(n))),
            (I::TST, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x39, Operand::AbsAdr(n))),
            (I::COM, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x3a, Operand::AbsAdr(n))),
            (I::LSL, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x3b, Operand::AbsAdr(n))),
            (I::LSR, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x3c, Operand::AbsAdr(n))),
            (I::ROL, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x3d, Operand::AbsAdr(n))),
            (I::ROR, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x3e, Operand::AbsAdr(n))),
            (I::ASR, OF::One(Atom::NumOrSym(n))) => Ok(op1(0x3f, Operand::AbsAdr(n))),
            (I::STX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x40, Operand::N(n)))
            }
            (I::STY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x41, Operand::N(n)))
            }
            (I::STSP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x42, Operand::N(n)))
            }
            (I::RTS, OF::None) => Ok(op0(0x43)),
            (I::RTI, OF::None) => Ok(op0(0x44)),
            (I::CLR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x45, Operand::N(n)))
            }
            (I::NEG, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x46, Operand::N(n)))
            }
            (I::INC, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x47, Operand::N(n)))
            }
            (I::DEC, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x48, Operand::N(n)))
            }
            (I::TST, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x49, Operand::N(n)))
            }
            (I::COM, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x4a, Operand::N(n)))
            }
            (I::LSL, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x4b, Operand::N(n)))
            }
            (I::LSR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x4c, Operand::N(n)))
            }
            (I::ROL, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x4d, Operand::N(n)))
            }
            (I::ROR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x4e, Operand::N(n)))
            }
            (I::ASR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0x4f, Operand::N(n)))
            }
            (I::STX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x50, Operand::N(n)))
            }
            (I::STY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x51, Operand::N(n)))
            }
            (I::STSP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x52, Operand::N(n)))
            }
            (I::JMP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x53, Operand::N(n)))
            }
            (I::JSR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x54, Operand::N(n)))
            }
            (I::CLR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x55, Operand::N(n)))
            }
            (I::NEG, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x56, Operand::N(n)))
            }
            (I::INC, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x57, Operand::N(n)))
            }
            (I::DEC, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x58, Operand::N(n)))
            }
            (I::TST, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x59, Operand::N(n)))
            }
            (I::COM, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x5a, Operand::N(n)))
            }
            (I::LSL, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x5b, Operand::N(n)))
            }
            (I::LSR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x5c, Operand::N(n)))
            }
            (I::ROL, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x5d, Operand::N(n)))
            }
            (I::ROR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x5e, Operand::N(n)))
            }
            (I::ASR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0x5f, Operand::N(n)))
            }
            (I::STX, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x60))
            }
            (I::STY, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x61))
            }
            (I::STSP, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x62))
            }
            (I::JMP, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x63))
            }
            (I::JSR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x64))
            }
            (I::CLR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x65))
            }
            (I::NEG, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x66))
            }
            (I::INC, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x67))
            }
            (I::DEC, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x68))
            }
            (I::TST, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x69))
            }
            (I::COM, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x6a))
            }
            (I::LSL, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x6b))
            }
            (I::LSR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x6c))
            }
            (I::ROL, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x6d))
            }
            (I::ROR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x6e))
            }
            (I::ASR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0x6f))
            }
            (I::STX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x70, Operand::N(n)))
            }
            (I::STY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x71, Operand::N(n)))
            }
            (I::STSP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x72, Operand::N(n)))
            }
            (I::JMP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x73, Operand::N(n)))
            }
            (I::JSR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x74, Operand::N(n)))
            }
            (I::CLR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x75, Operand::N(n)))
            }
            (I::NEG, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x76, Operand::N(n)))
            }
            (I::INC, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x77, Operand::N(n)))
            }
            (I::DEC, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x78, Operand::N(n)))
            }
            (I::TST, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x79, Operand::N(n)))
            }
            (I::COM, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x7a, Operand::N(n)))
            }
            (I::LSL, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x7b, Operand::N(n)))
            }
            (I::LSR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x7c, Operand::N(n)))
            }
            (I::ROL, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x7d, Operand::N(n)))
            }
            (I::ROR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x7e, Operand::N(n)))
            }
            (I::ASR, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0x7f, Operand::N(n)))
            }
            (I::STX, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x80))
            }
            (I::STY, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x81))
            }
            (I::STSP, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x82))
            }
            (I::JMP, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x83))
            }
            (I::JSR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x84))
            }
            (I::CLR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x85))
            }
            (I::NEG, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x86))
            }
            (I::INC, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x87))
            }
            (I::DEC, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x88))
            }
            (I::TST, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x89))
            }
            (I::COM, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x8a))
            }
            (I::LSL, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x8b))
            }
            (I::LSR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x8c))
            }
            (I::ROL, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x8d))
            }
            (I::ROR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x8e))
            }
            (I::ASR, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0x8f))
            }
            (I::LDX, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x90, Operand::Imm(n))),
            (I::LDY, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x91, Operand::Imm(n))),
            (I::LDSP, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x92, Operand::Imm(n))),
            (I::SBCA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x93, Operand::Imm(n))),
            (I::SUBA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x94, Operand::Imm(n))),
            (I::ADCA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x95, Operand::Imm(n))),
            (I::ADDA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x96, Operand::Imm(n))),
            (I::CMPA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x97, Operand::Imm(n))),
            (I::BITA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x98, Operand::Imm(n))),
            (I::ANDA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x99, Operand::Imm(n))),
            (I::ORA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x9a, Operand::Imm(n))),
            (I::EORA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x9b, Operand::Imm(n))),
            (I::CMPX, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x9c, Operand::Imm(n))),
            (I::CMPY, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x9d, Operand::Imm(n))),
            (I::CMPSP, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0x9e, Operand::Imm(n))),
            (I::EXG, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::CC))) => {
                Ok(op0(0x9f))
            }
            (I::LDX, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa0, Operand::AbsAdr(n))),
            (I::LDY, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa1, Operand::AbsAdr(n))),
            (I::LDSP, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa2, Operand::AbsAdr(n))),
            (I::SBCA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa3, Operand::AbsAdr(n))),
            (I::SUBA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa4, Operand::AbsAdr(n))),
            (I::ADCA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa5, Operand::AbsAdr(n))),
            (I::ADDA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa6, Operand::AbsAdr(n))),
            (I::CMPA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa7, Operand::AbsAdr(n))),
            (I::BITA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa8, Operand::AbsAdr(n))),
            (I::ANDA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xa9, Operand::AbsAdr(n))),
            (I::ORA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xaa, Operand::AbsAdr(n))),
            (I::EORA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xab, Operand::AbsAdr(n))),
            (I::CMPX, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xac, Operand::AbsAdr(n))),
            (I::CMPY, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xad, Operand::AbsAdr(n))),
            (I::CMPSP, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xae, Operand::AbsAdr(n))),
            (I::EXG, OF::Two(Atom::Reg(NamedLiteral::X), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0xaf))
            }
            (I::LDX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb0, Operand::N(n)))
            }
            (I::LDY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb1, Operand::N(n)))
            }
            (I::LDSP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb2, Operand::N(n)))
            }
            (I::SBCA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb3, Operand::N(n)))
            }
            (I::SUBA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb4, Operand::N(n)))
            }
            (I::ADCA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb5, Operand::N(n)))
            }
            (I::ADDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb6, Operand::N(n)))
            }
            (I::CMPA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb7, Operand::N(n)))
            }
            (I::BITA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb8, Operand::N(n)))
            }
            (I::ANDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xb9, Operand::N(n)))
            }
            (I::ORA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xba, Operand::N(n)))
            }
            (I::EORA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xbb, Operand::N(n)))
            }
            (I::CMPX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xbc, Operand::N(n)))
            }
            (I::CMPY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xbd, Operand::N(n)))
            }
            (I::LEASP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xbe, Operand::N(n)))
            }
            (I::EXG, OF::Two(Atom::Reg(NamedLiteral::X), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op0(0xbf))
            }
            (I::LDX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc0, Operand::N(n)))
            }
            (I::LDY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc1, Operand::N(n)))
            }
            (I::LDSP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc2, Operand::N(n)))
            }
            (I::SBCA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc3, Operand::N(n)))
            }
            (I::SUBA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc4, Operand::N(n)))
            }
            (I::ADCA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc5, Operand::N(n)))
            }
            (I::ADDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc6, Operand::N(n)))
            }
            (I::CMPA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc7, Operand::N(n)))
            }
            (I::BITA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc8, Operand::N(n)))
            }
            (I::ANDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xc9, Operand::N(n)))
            }
            (I::ORA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xca, Operand::N(n)))
            }
            (I::EORA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xcb, Operand::N(n)))
            }
            (I::LEAX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xcc, Operand::N(n)))
            }
            (I::LEAY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xcd, Operand::N(n)))
            }
            (I::LEASP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xce, Operand::N(n)))
            }
            (I::EXG, OF::Two(Atom::Reg(NamedLiteral::Y), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op0(0xcf))
            }
            (I::LDX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd0, Operand::N(n)))
            }
            (I::LDY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd1, Operand::N(n)))
            }
            (I::LDSP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd2, Operand::N(n)))
            }
            (I::SBCA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd3, Operand::N(n)))
            }
            (I::SUBA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd4, Operand::N(n)))
            }
            (I::ADCA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd5, Operand::N(n)))
            }
            (I::ADDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd6, Operand::N(n)))
            }
            (I::CMPA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd7, Operand::N(n)))
            }
            (I::BITA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd8, Operand::N(n)))
            }
            (I::ANDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xd9, Operand::N(n)))
            }
            (I::ORA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xda, Operand::N(n)))
            }
            (I::EORA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xdb, Operand::N(n)))
            }
            (I::LEAX, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xdc, Operand::N(n)))
            }
            (I::LEAY, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xdd, Operand::N(n)))
            }
            (I::LEASP, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xde, Operand::N(n)))
            }
            (I::STA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xe1, Operand::AbsAdr(n))),
            (I::STA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xe2, Operand::N(n)))
            }
            (I::STA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xe3, Operand::N(n)))
            }
            (I::STA, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0xe4))
            }
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::XPlus))) => Ok(op0(0xe5)),
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::XMinus))) => Ok(op0(0xe6)),
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::PlusX))) => Ok(op0(0xe7)),
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::MinusX))) => Ok(op0(0xe8)),
            (I::STA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xe9, Operand::N(n)))
            }
            (I::STA, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0xea))
            }
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::YPlus))) => Ok(op0(0xeb)),
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::YMinus))) => Ok(op0(0xec)),
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::PlusY))) => Ok(op0(0xed)),
            (I::STA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::MinusY))) => Ok(op0(0xee)),
            (I::LDA, OF::Imm1(Atom::NumOrSym(n))) => Ok(op1(0xf0, Operand::Imm(n))),
            (I::LDA, OF::One(Atom::NumOrSym(n))) => Ok(op1(0xf1, Operand::AbsAdr(n))),
            (I::LDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::SP))) => {
                Ok(op1(0xf2, Operand::N(n)))
            }
            (I::LDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::X))) => {
                Ok(op1(0xf3, Operand::N(n)))
            }
            (I::LDA, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::X))) => {
                Ok(op0(0xf4))
            }
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::XPlus))) => Ok(op0(0xf5)),
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::XMinus))) => Ok(op0(0xf6)),
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::PlusX))) => Ok(op0(0xf7)),
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::MinusX))) => Ok(op0(0xf8)),
            (I::LDA, OF::Two(Atom::NumOrSym(n), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op1(0xf9, Operand::N(n)))
            }
            (I::LDA, OF::Two(Atom::Reg(NamedLiteral::A), Atom::Reg(NamedLiteral::Y))) => {
                Ok(op0(0xfa))
            }
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::YPlus))) => Ok(op0(0xfb)),
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::YMinus))) => Ok(op0(0xfc)),
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::PlusY))) => Ok(op0(0xfd)),
            (I::LDA, OF::Two(Atom::None, Atom::Reg(NamedLiteral::MinusY))) => Ok(op0(0xfe)),
            _ => Err(self.err(
                "Invalid operand form for instruction".to_owned(),
                self.prev().span.start..self.curr().span.end,
            )),
        }?;

        Ok(AsmInstruction {
            opcode: res.0,
            operands: res.1,
            span: start..end,
        })
    }

    fn parse_operands(&mut self) -> Result<OperandForm, ParseError> {
        use TokenKind as TK;

        match self.curr().kind {
            TK::ImmediatePrefix => {
                self.advance();
                let op1 = self.parse_atom()?;
                Ok(OperandForm::Imm1(op1))
            }
            TK::NamedLiteral | TK::NumberLiteral | TK::Sym => {
                let op1 = self.parse_atom()?;
                match self.curr().kind {
                    TK::Comma => {
                        self.advance();
                        let op2 = self.parse_atom()?;
                        Ok(OperandForm::Two(op1, op2))
                    }
                    _ => Ok(OperandForm::One(op1)),
                }
            }
            TK::Comma => {
                self.advance();
                let op = self.parse_atom()?;
                Ok(OperandForm::Two(Atom::None, op))
            }
            _ => Ok(OperandForm::None),
        }
    }

    fn parse_atom(&mut self) -> Result<Atom, ParseError> {
        let val = match self.curr().kind {
            TokenKind::NamedLiteral => {
                let name_lit = self.curr().value.expect_named_literal();
                Ok(Atom::Reg(name_lit))
            }
            TokenKind::NumberLiteral => {
                let num_lit = self.curr().value.expect_number_literal();
                Ok(Atom::NumOrSym(NumOrSym::Num(num_lit)))
            }
            TokenKind::Sym => {
                let sym = self.curr().value.expect_sym();
                Ok(Atom::NumOrSym(NumOrSym::Sym(sym.0.to_owned())))
            }
            _ => Err(self.err("Expected operand".to_string(), self.curr_span())),
        }?;

        self.advance();
        Ok(val)
    }
}
