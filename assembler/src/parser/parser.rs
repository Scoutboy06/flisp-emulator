use std::ops::Range;

use super::{
    instruction_selection::{Operand, select_instruction},
    syntax::{Atom, Expression, OperandForm},
};

use ariadne::{Label, Report, ReportKind, Source};

use crate::lexer::{
    Lexer,
    directive::{Directive, parse_directive as identify_directive},
    instruction::parse_instruction as identify_instruction,
    parse_named_literal,
    token::{Token, TokenKind},
};

#[derive(Debug)]
pub struct ProgramAST {
    pub lines: Vec<AsmLine>,
}

#[derive(Debug)]
pub enum AsmLine {
    Label {
        name: String,
        span: Range<usize>,
    },
    Instruction {
        label: Option<String>,
        instr: AsmInstruction,
    },
    Directive {
        label: Option<String>,
        dir: AsmDirective,
    },
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
        self.advance();

        let mut lines: Vec<AsmLine> = Vec::new();

        while self.curr().kind != TokenKind::Eof {
            if self.curr().kind == TokenKind::Newline {
                self.advance();
                continue;
            }

            lines.push(self.parse_statement()?);
            self.expect_line_end()?;
        }

        Ok(ProgramAST { lines })
    }

    fn parse_statement(&mut self) -> Result<AsmLine, ParseError> {
        let statement_start = self.curr().span.start;
        let label = self.parse_optional_label()?;

        if matches!(self.curr().kind, TokenKind::Newline | TokenKind::Eof) {
            return match label {
                Some(name) => Ok(AsmLine::Label {
                    name,
                    span: statement_start..self.prev().span.end,
                }),
                None => Err(self.err("Expected instruction or directive".into(), self.curr_span())),
            };
        }

        let identifier = if self.curr().kind == TokenKind::Identifier {
            self.curr().value.expect_identifier()
        } else {
            return Err(self.err("Expected instruction or directive".into(), self.curr_span()));
        };

        if identify_instruction(identifier).is_some() {
            let instr = self.parse_instruction()?;
            Ok(AsmLine::Instruction { label, instr })
        } else if identify_directive(identifier).is_some() {
            let dir = self.parse_directive()?;
            Ok(AsmLine::Directive { label, dir })
        } else {
            Err(self.err(
                format!("Unknown instruction or directive `{identifier}`"),
                self.curr_span(),
            ))
        }
    }

    fn parse_optional_label(&mut self) -> Result<Option<String>, ParseError> {
        if self.curr().kind != TokenKind::Identifier {
            return Ok(None);
        }

        let identifier = self.curr().value.expect_identifier();
        if identify_instruction(identifier).is_some() || identify_directive(identifier).is_some() {
            return Ok(None);
        }

        let label = identifier.to_owned();
        self.advance();
        if self.curr().kind == TokenKind::Colon {
            self.advance();
        }
        Ok(Some(label))
    }

    fn expect_line_end(&mut self) -> Result<(), ParseError> {
        match self.curr().kind {
            TokenKind::Newline => {
                self.advance();
                Ok(())
            }
            TokenKind::Eof => Ok(()),
            _ => Err(self.err("Expected end of line".into(), self.curr_span())),
        }
    }

    fn parse_directive(&mut self) -> Result<AsmDirective, ParseError> {
        let start_pos = self.curr().span.start;
        let name = self.curr().value.expect_identifier();
        match identify_directive(name).expect("directive checked before parsing") {
            Directive::Org => {
                self.advance();
                let span = start_pos..self.curr().span.end;
                match self.curr().kind {
                    TokenKind::NumberLiteral | TokenKind::Identifier => Ok(AsmDirective {
                        span,
                        name: Directive::Org,
                        args: vec![self.parse_atom().unwrap()],
                    }),
                    _ => Err(self.err("Expected number or symbol".into(), span)),
                }
            }
            Directive::Equ => {
                self.advance();
                if matches!(
                    self.curr().kind,
                    TokenKind::NumberLiteral | TokenKind::Identifier
                ) {
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

                while let TokenKind::NumberLiteral | TokenKind::Identifier = self.curr().kind {
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
        let name = self.curr().value.expect_identifier();
        let ins = identify_instruction(name).expect("instruction checked before parsing");
        self.advance(); // Consume instruction token
        let ops = self.parse_operands()?;
        let end = self.prev().span.end;

        let error_span = self.prev().span.start..self.curr().span.end;
        let res = select_instruction(ins, ops).ok_or_else(|| {
            self.err(
                "Invalid operand form for instruction".to_owned(),
                error_span,
            )
        })?;

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
            TK::Identifier | TK::NumberLiteral => {
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
        let span = self.curr_span();
        let val = match self.curr().kind {
            TokenKind::NumberLiteral => {
                let value = self.curr().value.expect_number_literal();
                Ok(Atom::Expr(Expression::Number { value, span }))
            }
            TokenKind::Identifier => {
                let identifier = self.curr().value.expect_identifier();
                if let Some(register) = parse_named_literal(identifier) {
                    Ok(Atom::Reg(register))
                } else {
                    Ok(Atom::Expr(Expression::Symbol {
                        name: identifier.to_owned(),
                        span,
                    }))
                }
            }
            _ => Err(self.err("Expected operand".to_string(), self.curr_span())),
        }?;

        self.advance();
        Ok(val)
    }
}
