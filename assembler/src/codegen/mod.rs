use std::{collections::HashMap, ops::Range};

use ariadne::{Color, Label, Report, ReportKind, Source};
use srec::{Address16, Data, Record};

use crate::{
    lexer::directive::Directive,
    parser::{
        AsmDirective, AsmInstruction, AsmLine, Atom, Expression, Operand, ParseError, Parser,
        ProgramAST,
    },
};

#[derive(Debug)]
pub enum AssembleError {
    Parse(ParseError),
    CircularDefinition { edges: Vec<DependencyEdge> },
    OverflowFromInstruction(AsmInstruction),
    OverflowFromDirective(AsmDirective),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub reference_span: Range<usize>,
}

impl AssembleError {
    pub fn report_on(&self, file_name: &str, src: &str) {
        let report = self.build_report(file_name);
        report.eprint((file_name, Source::from(src))).unwrap();
    }

    pub fn build_report<'a>(&'a self, file_name: &'a str) -> Report<'a, (&'a str, Range<usize>)> {
        match self {
            AssembleError::Parse(e) => e.build_report(file_name),
            AssembleError::CircularDefinition { edges } => {
                let closing = edges
                    .last()
                    .expect("a dependency cycle has at least one edge");
                let mut report = Report::build(
                    ReportKind::Error,
                    (file_name, closing.reference_span.to_owned()),
                )
                .with_message("Circular symbol definition");

                for (index, edge) in edges.iter().enumerate() {
                    let closes_cycle = index + 1 == edges.len();
                    let message = if closes_cycle {
                        format!("{} depends on {}, completing the cycle", edge.from, edge.to)
                    } else {
                        format!("{} depends on {}", edge.from, edge.to)
                    };
                    report = report.with_label(
                        Label::new((file_name, edge.reference_span.to_owned()))
                            .with_color(if closes_cycle {
                                Color::Red
                            } else {
                                Color::Yellow
                            })
                            .with_message(message),
                    );
                }

                let mut path: Vec<&str> = edges.iter().map(|edge| edge.from.as_str()).collect();
                path.push(closing.to.as_str());
                report
                    .with_note(format!("dependency cycle: {}", path.join(" -> ")))
                    .finish()
            }
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

    let mut memory = Memory::default();

    for line in ast.lines {
        match line {
            AsmLine::Label { .. } => {}
            AsmLine::Instruction { label: _, instr } => {
                memory
                    .write_byte(instr.opcode)
                    .map_err(|_| AssembleError::OverflowFromInstruction(instr.to_owned()))?;
                for operand in instr.operands.iter() {
                    match operand {
                        Operand::Imm(val)
                        | Operand::AbsAdr(val)
                        | Operand::RelAdr(val)
                        | Operand::N(val) => match val {
                            Expression::Number { value: n, .. } => {
                                memory.write_byte(*n).map_err(|_| {
                                    AssembleError::OverflowFromInstruction(instr.to_owned())
                                })?;
                            }
                            Expression::Symbol { name: sym, span } => {
                                let val = symbols.get(sym.as_str()).ok_or_else(|| {
                                    AssembleError::Parse(ParseError::new(
                                        format!("Undefined symbol: {}", sym),
                                        span.to_owned(),
                                    ))
                                })?;
                                memory.write_byte(*val).map_err(|_| {
                                    AssembleError::OverflowFromInstruction(instr.to_owned())
                                })?;
                            }
                        },
                        Operand::Reg(_) => { /* Not written to memory */ }
                    }
                }
            }
            AsmLine::Directive { label: _, dir } => match dir.name {
                Directive::Org => match dir.args.first() {
                    Some(Atom::Expr(n_or_sym)) => match n_or_sym {
                        Expression::Number { value: n, .. } => memory.set_pc(*n),
                        Expression::Symbol { name: sym, span } => {
                            let new_addr = symbols.get(sym).ok_or_else(|| {
                                AssembleError::Parse(ParseError::new(
                                    format!("Undefined symbol: {}", sym),
                                    span.to_owned(),
                                ))
                            })?;
                            memory.set_pc(*new_addr);
                        }
                    },
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
                            Atom::Expr(n_or_sym) => match n_or_sym {
                                Expression::Number { value: n, .. } => {
                                    memory.write_byte(*n).map_err(|_| {
                                        dbg!(AssembleError::OverflowFromDirective(dir.clone()))
                                    })?
                                }
                                Expression::Symbol { name: sym, span } => {
                                    let val = symbols.get(sym.as_str()).ok_or_else(|| {
                                        AssembleError::Parse(ParseError::new(
                                            format!("Undefined symbol: {}", sym),
                                            span.to_owned(),
                                        ))
                                    })?;
                                    memory.write_byte(*val).map_err(|_| {
                                        dbg!(AssembleError::OverflowFromDirective(dir.clone()))
                                    })?
                                }
                            },
                            _ => unreachable!(),
                        }
                    }
                }
                Directive::Equ => {}
                _ => todo!(),
            },
        }
    }

    Ok(*memory.get_data())
}

#[derive(Debug, Clone, Copy)]
enum ResolutionState {
    Resolving,
    Resolved(u8),
}

fn collect_symbols(ast: &ProgramAST) -> Result<HashMap<String, u8>, AssembleError> {
    let mut definitions = HashMap::new();

    // Constants are collected before layout so EQU aliases can refer forward.
    for line in &ast.lines {
        let AsmLine::Directive {
            label,
            dir:
                AsmDirective {
                    name: Directive::Equ,
                    args,
                    span,
                },
        } = line
        else {
            continue;
        };

        let name = label.as_ref().ok_or_else(|| {
            AssembleError::Parse(ParseError::new(
                "EQU directives require a symbol definition",
                span.to_owned(),
            ))
        })?;
        let expression = match args.first() {
            Some(Atom::Expr(expression)) => expression.to_owned(),
            _ => {
                return Err(AssembleError::Parse(ParseError::new(
                    "EQU directive requires a value",
                    span.to_owned(),
                )));
            }
        };
        if definitions.insert(name.to_owned(), expression).is_some() {
            return Err(AssembleError::Parse(ParseError::new(
                format!("Duplicate symbol: {}", name),
                span.to_owned(),
            )));
        }
    }

    let mut symbols = HashMap::new();
    let mut states = HashMap::new();
    let mut memory = Memory::default();

    for line in &ast.lines {
        match line {
            AsmLine::Label { name, span } => {
                define_address(&mut symbols, &definitions, name, memory.get_pc(), span)?;
            }
            AsmLine::Instruction { label, instr } => {
                if let Some(name) = label {
                    define_address(
                        &mut symbols,
                        &definitions,
                        name,
                        memory.get_pc(),
                        &instr.span,
                    )?;
                }
                memory
                    .inc_pc(instr.size())
                    .map_err(|_| AssembleError::OverflowFromInstruction(instr.to_owned()))?;
            }
            AsmLine::Directive { label, dir } if dir.name == Directive::Equ => {}
            AsmLine::Directive { label, dir } => {
                if let Some(name) = label {
                    define_address(&mut symbols, &definitions, name, memory.get_pc(), &dir.span)?;
                }

                match dir.name {
                    Directive::Org => match dir.args.first() {
                        Some(Atom::Expr(expression)) => {
                            let value = resolve_expression(
                                expression,
                                &definitions,
                                &symbols,
                                &mut states,
                                &mut Vec::new(),
                                &mut Vec::new(),
                            )?;
                            memory.set_pc(value);
                        }
                        _ => {
                            return Err(AssembleError::Parse(ParseError::new(
                                "ORG directive requires an address argument",
                                dir.span.to_owned(),
                            )));
                        }
                    },
                    Directive::Fcb => {
                        memory
                            .inc_pc(dir.args.len() as u8)
                            .map_err(|_| AssembleError::OverflowFromDirective(dir.to_owned()))?;
                    }
                    Directive::Equ => unreachable!(),
                    Directive::Fcs => todo!(),
                    Directive::Rmb => todo!(),
                }
            }
        }
    }

    let mut definition_names: Vec<_> = definitions.keys().cloned().collect();
    definition_names.sort();
    for name in definition_names {
        let value = resolve_symbol(
            &name,
            &definitions,
            &symbols,
            &mut states,
            &mut Vec::new(),
            &mut Vec::new(),
        )?;
        symbols.insert(name, value);
    }

    Ok(symbols)
}

fn define_address(
    symbols: &mut HashMap<String, u8>,
    definitions: &HashMap<String, Expression>,
    name: &str,
    address: u8,
    span: &Range<usize>,
) -> Result<(), AssembleError> {
    if symbols.contains_key(name) || definitions.contains_key(name) {
        return Err(AssembleError::Parse(ParseError::new(
            format!("Duplicate symbol: {}", name),
            span.to_owned(),
        )));
    }
    symbols.insert(name.to_owned(), address);
    Ok(())
}

fn resolve_expression(
    expression: &Expression,
    definitions: &HashMap<String, Expression>,
    addresses: &HashMap<String, u8>,
    states: &mut HashMap<String, ResolutionState>,
    path: &mut Vec<String>,
    edges: &mut Vec<DependencyEdge>,
) -> Result<u8, AssembleError> {
    match expression {
        Expression::Number { value, .. } => Ok(*value),
        Expression::Symbol { name, span } => {
            if let Some(value) = addresses.get(name) {
                return Ok(*value);
            }
            if !definitions.contains_key(name) {
                return Err(AssembleError::Parse(ParseError::new(
                    format!("Undefined symbol: {}", name),
                    span.to_owned(),
                )));
            }
            resolve_symbol(name, definitions, addresses, states, path, edges)
        }
    }
}

fn resolve_symbol(
    name: &str,
    definitions: &HashMap<String, Expression>,
    addresses: &HashMap<String, u8>,
    states: &mut HashMap<String, ResolutionState>,
    path: &mut Vec<String>,
    edges: &mut Vec<DependencyEdge>,
) -> Result<u8, AssembleError> {
    match states.get(name) {
        Some(ResolutionState::Resolved(value)) => return Ok(*value),
        Some(ResolutionState::Resolving) => {
            let cycle_start = edges.iter().position(|edge| edge.from == name).unwrap_or(0);
            return Err(AssembleError::CircularDefinition {
                edges: edges[cycle_start..].to_vec(),
            });
        }
        None => {}
    }

    states.insert(name.to_owned(), ResolutionState::Resolving);
    path.push(name.to_owned());

    let expression = definitions
        .get(name)
        .expect("only defined constants are resolved");
    let value = match expression {
        Expression::Number { value, .. } => *value,
        Expression::Symbol {
            name: dependency,
            span,
        } => {
            edges.push(DependencyEdge {
                from: name.to_owned(),
                to: dependency.to_owned(),
                reference_span: span.to_owned(),
            });
            let value =
                resolve_expression(expression, definitions, addresses, states, path, edges)?;
            edges.pop();
            value
        }
    };

    path.pop();
    states.insert(name.to_owned(), ResolutionState::Resolved(value));
    Ok(value)
}

pub fn emit_s19(mem: &[u8; 256]) -> String {
    // Each record holds up to 30 bytes of equential data.
    //
    // If there are gaps in the memory (2 or more null bytes in row),
    // separate records are created.
    //
    // A separate S9 record is created for the start address stored at
    // memory location 0xFF, even if that memory is set via a S1 record already.

    let mut records: Vec<Record> = Vec::new();

    let mut null_count = 0;
    let mut seq_start: Option<u8> = None;
    for addr in 0..=255_u8 {
        let byte = mem[addr as usize];
        if byte == 0 {
            null_count += 1;
            if null_count == 2 {
                // End of a sequential data block
                if let Some(start) = seq_start {
                    let end = addr - 2;
                    records.push(create_s1_record(mem, start, end));
                    seq_start = None;
                }
            }
        } else {
            if null_count >= 2 || seq_start.is_none() {
                // Start of a new sequential data block
                seq_start = Some(addr);
            } else if seq_start.is_some_and(|s| addr - s == 30) {
                let start = seq_start.unwrap();
                records.push(create_s1_record(mem, start, addr - 1));
                seq_start = Some(addr);
            }
            null_count = 0;
        }
    }

    if let Some(start) = seq_start {
        let end = 255_u8;
        records.push(create_s1_record(mem, start, end));
    }

    let start_addr = mem[255];
    if start_addr != 0 {
        records.push(Record::S9(Address16(start_addr as u16)));
    }

    srec::generate_srec_file(&records)
}

fn create_s1_record(mem: &[u8; 256], start: u8, end: u8) -> Record {
    let data = mem[start as usize..=end as usize].to_owned();
    Record::S1(Data {
        address: Address16(start as u16),
        data,
    })
}

pub fn emit_fmem(mem: &[u8; 256], file_name: &str) -> String {
    let mut out = format!("File: {file_name}\n\n # ClearAllMemory\n # ClearAllRegisters");

    for (adr, byte) in mem.iter().enumerate() {
        if *byte != 0 {
            out.push_str(&format!("\n #setMemory  {:02X}={:02X}", adr, byte))
        }
    }

    out
}
