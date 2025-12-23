use std::{collections::HashMap, ops::Range};

use ariadne::{Label, Report, ReportKind, Source};
use srec::Record;

use crate::{
    lexer::directive::Directive,
    parser::{
        AsmDirective, AsmInstruction, AsmLine, Atom, Operand, ParseError, Parser, ProgramAST,
    },
};

#[derive(Debug)]
pub enum AssembleError {
    Parse(ParseError),
    OverflowFromInstruction(AsmInstruction),
    OverflowFromDirective(AsmDirective),
}

impl AssembleError {
    pub fn report_on(&self, file_name: &str, src: &str) {
        let report = self.build_report(file_name);
        report.eprint((file_name, Source::from(src))).unwrap();
    }

    pub fn build_report<'a>(&'a self, file_name: &'a str) -> Report<'a, (&'a str, Range<usize>)> {
        match self {
            AssembleError::Parse(e) => e.build_report(file_name),
            AssembleError::OverflowFromInstruction(ins) => {
                Report::build(ReportKind::Error, (file_name, ins.span.to_owned()))
                    .with_message("Memory overflow occurred while assembling instruction")
                    .with_label(
                        Label::new((file_name, ins.span.to_owned()))
                            .with_message(format!("this instruction")),
                    )
                    .finish()
            }
            AssembleError::OverflowFromDirective(dir) => {
                Report::build(ReportKind::Error, (file_name, dir.span.to_owned()))
                    .with_message("Memory overflow occurred while assembling directive")
                    .with_label(
                        Label::new((file_name, dir.span.to_owned()))
                            .with_message(format!("this directive")),
                    )
                    .finish()
            }
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    data: [u8; 256],
    pc: u16,
}

#[derive(Debug)]
pub enum MemoryError {
    Overflow,
    OutOfBounds(usize),
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: [0u8; 256],
            pc: 0,
        }
    }
}

impl Memory {
    pub fn write_byte(&mut self, byte: u8) -> Result<(), MemoryError> {
        let addr = self.pc as usize;
        if addr >= self.data.len() {
            return Err(MemoryError::OutOfBounds(addr));
        }
        self.data[addr] = byte;

        // Update the program counter and check for overflow
        let (new_pc, overflow) = self.pc.overflowing_add(1);
        self.pc = new_pc;

        // Overflow is only an error if it happens after writing to the last valid address
        if overflow && self.pc != 0 {
            return Err(MemoryError::Overflow);
        }

        Ok(())
    }

    pub fn set_pc(&mut self, new_pc: u8) {
        self.pc = new_pc as u16;
    }

    pub fn get_pc(&self) -> u8 {
        self.pc as u8
    }

    pub fn inc_pc(&mut self, inc: u8) -> Result<(), MemoryError> {
        let (new_pc, overflow) = self.pc.overflowing_add(inc as u16);
        self.pc = new_pc;

        if overflow && self.pc != 0 {
            return Err(MemoryError::Overflow);
        }
        Ok(())
    }

    pub fn get_data(&self) -> &[u8; 256] {
        &self.data
    }
}

pub fn assemble(src: &str, file_path: String) -> Result<[u8; 256], AssembleError> {
    let ast = Parser::from_source(src)
        .with_source_name(file_path)
        .parse()
        .map_err(AssembleError::Parse)?;

    let symbols = collect_symbols(&ast)?;
    dbg!(&symbols);

    let mut memory = Memory::default();

    for line in ast.lines {
        match line {
            AsmLine::Instruction(ins) => {
                memory
                    .write_byte(ins.opcode)
                    .map_err(|_| AssembleError::OverflowFromInstruction(ins.to_owned()))?;
                for operand in ins.operands.iter() {
                    match operand {
                        Operand::Imm(val)
                        | Operand::AbsAdr(val)
                        | Operand::RelAdr(val)
                        | Operand::N(val) => {
                            memory.write_byte(*val).map_err(|_| {
                                AssembleError::OverflowFromInstruction(ins.to_owned())
                            })?;
                        }
                        Operand::Reg(_) => { /* Not written to memory */ }
                    }
                }
            }
            AsmLine::Directive(dir) => match dir.name {
                Directive::Org => match dir.args.first() {
                    Some(Atom::Number(n)) => memory.set_pc(*n),
                    Some(Atom::Symbol(sym)) => {
                        let new_addr = symbols.get(sym).ok_or_else(|| {
                            AssembleError::Parse(ParseError::new(
                                format!("Undefined symbol: {}", sym),
                                dir.span,
                            ))
                        })?;
                        memory.set_pc(*new_addr);
                    }
                    _ => {
                        return Err(AssembleError::Parse(ParseError::new(
                            "ORG directive requires an address argument".to_string(),
                            dir.span,
                        )));
                    }
                },
                Directive::Fcb => {
                    for arg in dir.args.iter() {
                        match arg {
                            Atom::Number(n) => memory.write_byte(*n).map_err(|_| {
                                dbg!(AssembleError::OverflowFromDirective(dir.clone()))
                            })?,
                            Atom::Symbol(sym) => {
                                let val = symbols.get(sym.as_str()).ok_or_else(|| {
                                    AssembleError::Parse(ParseError::new(
                                        format!("Undefined symbol: {}", sym),
                                        dir.span.clone(),
                                    ))
                                })?;
                                memory.write_byte(*val).map_err(|_| {
                                    dbg!(AssembleError::OverflowFromDirective(dir.clone()))
                                })?;
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => todo!(),
            },
            AsmLine::Symbol(_) => { /* Symbols are already collected */ }
        }
    }

    Ok(*memory.get_data())
}

fn collect_symbols(ast: &ProgramAST) -> Result<HashMap<String, u8>, AssembleError> {
    let mut symbols: HashMap<String, u8> = HashMap::new();

    let mut memory = Memory::default();

    for line in &ast.lines {
        match line {
            AsmLine::Symbol(sym) => {
                if symbols.contains_key(&sym.name) {
                    return Err(AssembleError::Parse(ParseError::new(
                        format!("Duplicate symbol: {}", sym.name),
                        sym.span.to_owned(),
                    )));
                }
                symbols.insert(sym.name.to_owned(), memory.get_pc());
            }
            AsmLine::Directive(dir) => match dir.name {
                Directive::Org => match dir.args.first() {
                    Some(Atom::Number(n)) => {
                        memory.set_pc(*n);
                    }
                    Some(Atom::Symbol(sym)) => {
                        let new_addr = symbols.get(sym).ok_or_else(|| {
                            AssembleError::Parse(ParseError::new(
                                format!("Undefined symbol: {}", sym),
                                dir.span.to_owned(),
                            ))
                        })?;
                        memory.set_pc(*new_addr);
                    }
                    _ => {
                        return Err(AssembleError::Parse(ParseError::new(
                            "ORG directive requires an address argument".to_string(),
                            dir.span.to_owned(),
                        )));
                    }
                },
                Directive::Equ => {
                    return Err(AssembleError::Parse(ParseError::new(
                        "EQU directives require a symbol definition".to_string(),
                        dir.span.to_owned(),
                    )));
                }
                Directive::Fcb => {
                    let size = dir.args.len() as u8;
                    memory
                        .inc_pc(size)
                        .map_err(|_| dbg!(AssembleError::OverflowFromDirective(dir.to_owned())))?;
                }
                Directive::Fcs => todo!(),
                Directive::Rmb => todo!(),
            },
            AsmLine::Instruction(ins) => {
                memory
                    .write_byte(ins.opcode)
                    .map_err(|_| AssembleError::OverflowFromInstruction(ins.to_owned()))?;
            }
        }
    }

    Ok(symbols)
}

pub fn emit_s19(mem: &[u8; 256]) -> String {
    let records: Vec<Record> = Vec::new();
    srec::generate_srec_file(&records)
}

pub fn emit_fmem(_mem: &[u8; 256]) -> String {
    todo!()
}
