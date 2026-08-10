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
    DuplicateSymbol {
        name: String,
        definition_spans: Vec<Range<usize>>,
    },
    CircularDefinition {
        edges: Vec<DependencyEdge>,
    },
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
            AssembleError::DuplicateSymbol {
                name,
                definition_spans,
            } => {
                let duplicate_span = definition_spans
                    .last()
                    .expect("duplicate symbols have at least two definitions");
                let mut report =
                    Report::build(ReportKind::Error, (file_name, duplicate_span.to_owned()))
                        .with_message(format!("Duplicate symbol `{name}`"));

                for (index, span) in definition_spans.iter().enumerate() {
                    let original = index == 0;
                    report = report.with_label(
                        Label::new((file_name, span.to_owned()))
                            .with_color(if original { Color::Yellow } else { Color::Red })
                            .with_message(if original {
                                format!("`{name}` was first defined here")
                            } else {
                                format!("duplicate definition of `{name}`")
                            }),
                    );
                }
                report.finish()
            }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWriteOrigin {
    pub addresses: Vec<u8>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblyWarning {
    MemoryWrap {
        span: Range<usize>,
    },
    MemoryOverwrite {
        addresses: Vec<u8>,
        original_writes: Vec<MemoryWriteOrigin>,
        overwrite_span: Range<usize>,
    },
}

impl AssemblyWarning {
    pub fn report_on(&self, file_name: &str, src: &str) {
        self.build_report(file_name)
            .eprint((file_name, Source::from(src)))
            .unwrap();
    }

    pub fn build_report<'a>(&'a self, file_name: &'a str) -> Report<'a, (&'a str, Range<usize>)> {
        match self {
            Self::MemoryWrap { span } => {
                Report::build(ReportKind::Warning, (file_name, span.to_owned()))
                    .with_message("Assembly wraps around the end of memory")
                    .with_label(
                        Label::new((file_name, span.to_owned()))
                            .with_color(Color::Yellow)
                            .with_message("emission continues at address $00"),
                    )
                    .finish()
            }
            Self::MemoryOverwrite {
                addresses,
                original_writes,
                overwrite_span,
            } => {
                let addresses = format_addresses(addresses);
                let mut report =
                    Report::build(ReportKind::Warning, (file_name, overwrite_span.to_owned()))
                        .with_message("Assembly overwrites initialized memory");
                for original in original_writes {
                    report = report.with_label(
                        Label::new((file_name, original.span.to_owned()))
                            .with_color(Color::Yellow)
                            .with_message(format!(
                                "{} first written here",
                                format_addresses(&original.addresses)
                            )),
                    );
                }
                report
                    .with_label(
                        Label::new((file_name, overwrite_span.to_owned()))
                            .with_color(Color::Red)
                            .with_message(format!("overwrites {addresses}")),
                    )
                    .finish()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssemblyOutput {
    memory: [u8; 256],
    initialized: [bool; 256],
    first_emitted: Option<u8>,
    warnings: Vec<AssemblyWarning>,
}

impl AssemblyOutput {
    pub fn memory(&self) -> &[u8; 256] {
        &self.memory
    }

    pub fn initialized(&self) -> &[bool; 256] {
        &self.initialized
    }

    pub fn warnings(&self) -> &[AssemblyWarning] {
        &self.warnings
    }
}

#[derive(Debug)]
pub struct Memory {
    data: [u8; 256],
    initialized: [bool; 256],
    first_write_spans: Vec<Option<Range<usize>>>,
    first_emitted: Option<u8>,
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
            initialized: [false; 256],
            first_write_spans: vec![None; 256],
            first_emitted: None,
            pc: 0,
        }
    }
}

impl Memory {
    pub fn write_byte(
        &mut self,
        byte: u8,
        source_span: &Range<usize>,
    ) -> Result<Option<Range<usize>>, MemoryError> {
        let addr = self.get_pc() as usize;
        let original_span = self.first_write_spans[addr].to_owned();
        self.data[addr] = byte;
        self.initialized[addr] = true;
        self.first_write_spans[addr].get_or_insert_with(|| source_span.to_owned());
        self.first_emitted.get_or_insert(addr as u8);
        self.pc = self.pc.wrapping_add(1);
        Ok(original_span)
    }

    pub fn set_pc(&mut self, new_pc: u8) {
        self.pc = new_pc as u16;
    }

    pub fn get_pc(&self) -> u8 {
        self.pc as u8
    }

    pub fn inc_pc(&mut self, inc: u8) -> Result<(), MemoryError> {
        self.pc = self.pc.wrapping_add(inc as u16);
        Ok(())
    }

    pub fn into_output(self, warnings: Vec<AssemblyWarning>) -> AssemblyOutput {
        AssemblyOutput {
            memory: self.data,
            initialized: self.initialized,
            first_emitted: self.first_emitted,
            warnings,
        }
    }
}

pub fn assemble(src: &str, file_path: String) -> Result<AssemblyOutput, AssembleError> {
    let ast = Parser::from_source(src)
        .with_source_name(file_path)
        .parse()
        .map_err(AssembleError::Parse)?;

    let symbols = collect_symbols(&ast)?;

    let mut memory = Memory::default();
    let mut warnings = Vec::new();

    for line in ast.lines {
        match line {
            AsmLine::Label { .. } => {}
            AsmLine::Instruction { label: _, instr } => {
                let mut overwritten = Vec::new();
                if memory.get_pc() as usize + instr.size() as usize > 256 {
                    warnings.push(AssemblyWarning::MemoryWrap {
                        span: instr.span.to_owned(),
                    });
                }
                write_emitted_byte(&mut memory, instr.opcode, &instr.span, &mut overwritten)
                    .map_err(|_| AssembleError::OverflowFromInstruction(instr.to_owned()))?;
                for operand in instr.operands.iter() {
                    match operand {
                        Operand::RelAdr(expression) => {
                            let target = resolve_emitted_expression(expression, &symbols)?;
                            let next_instruction = memory.get_pc().wrapping_add(1);
                            let offset = target.wrapping_sub(next_instruction);
                            write_emitted_byte(&mut memory, offset, &instr.span, &mut overwritten)
                                .map_err(|_| {
                                    AssembleError::OverflowFromInstruction(instr.to_owned())
                                })?;
                        }
                        Operand::Imm(expression)
                        | Operand::AbsAdr(expression)
                        | Operand::N(expression) => {
                            let value = resolve_emitted_expression(expression, &symbols)?;
                            write_emitted_byte(&mut memory, value, &instr.span, &mut overwritten)
                                .map_err(|_| {
                                AssembleError::OverflowFromInstruction(instr.to_owned())
                            })?;
                        }
                        Operand::Reg(_) => { /* Not written to memory */ }
                    }
                }
                if let Some(warning) = memory_overwrite_warning(overwritten, &instr.span) {
                    warnings.push(warning);
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
                    let mut overwritten = Vec::new();
                    if memory.get_pc() as usize + dir.args.len() > 256 {
                        warnings.push(AssemblyWarning::MemoryWrap {
                            span: dir.span.to_owned(),
                        });
                    }
                    for arg in dir.args.iter() {
                        match arg {
                            Atom::Expr(n_or_sym) => match n_or_sym {
                                Expression::Number { value: n, .. } => {
                                    write_emitted_byte(&mut memory, *n, &dir.span, &mut overwritten)
                                        .map_err(|_| {
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
                                    write_emitted_byte(
                                        &mut memory,
                                        *val,
                                        &dir.span,
                                        &mut overwritten,
                                    )
                                    .map_err(|_| {
                                        dbg!(AssembleError::OverflowFromDirective(dir.clone()))
                                    })?
                                }
                            },
                            _ => unreachable!(),
                        }
                    }
                    if let Some(warning) = memory_overwrite_warning(overwritten, &dir.span) {
                        warnings.push(warning);
                    }
                }
                Directive::Equ => {}
                _ => todo!(),
            },
        }
    }

    Ok(memory.into_output(warnings))
}

fn write_emitted_byte(
    memory: &mut Memory,
    byte: u8,
    source_span: &Range<usize>,
    overwritten: &mut Vec<(u8, Range<usize>)>,
) -> Result<(), MemoryError> {
    let address = memory.get_pc();
    if let Some(original_span) = memory.write_byte(byte, source_span)? {
        overwritten.push((address, original_span));
    }
    Ok(())
}

fn memory_overwrite_warning(
    overwritten: Vec<(u8, Range<usize>)>,
    overwrite_span: &Range<usize>,
) -> Option<AssemblyWarning> {
    if overwritten.is_empty() {
        return None;
    }

    let mut addresses = Vec::new();
    let mut original_writes: Vec<MemoryWriteOrigin> = Vec::new();
    for (address, span) in overwritten {
        if !addresses.contains(&address) {
            addresses.push(address);
        }
        if let Some(origin) = original_writes
            .iter_mut()
            .find(|origin| origin.span == span)
        {
            if !origin.addresses.contains(&address) {
                origin.addresses.push(address);
            }
        } else {
            original_writes.push(MemoryWriteOrigin {
                addresses: vec![address],
                span,
            });
        }
    }
    Some(AssemblyWarning::MemoryOverwrite {
        addresses,
        original_writes,
        overwrite_span: overwrite_span.to_owned(),
    })
}

fn format_addresses(addresses: &[u8]) -> String {
    let mut addresses = addresses.to_vec();
    addresses.sort_unstable();
    addresses.dedup();

    let mut ranges = Vec::new();
    let mut start = addresses[0];
    let mut end = start;
    for address in addresses.iter().copied().skip(1) {
        if end.checked_add(1) == Some(address) {
            end = address;
        } else {
            ranges.push((start, end));
            start = address;
            end = address;
        }
    }
    ranges.push((start, end));

    let formatted = ranges
        .into_iter()
        .map(|(start, end)| {
            if start == end {
                format!("${start:02X}")
            } else {
                format!("${start:02X}–${end:02X}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    if addresses.len() == 1 {
        format!("address {formatted}")
    } else {
        format!("addresses {formatted}")
    }
}

fn resolve_emitted_expression(
    expression: &Expression,
    symbols: &HashMap<String, u8>,
) -> Result<u8, AssembleError> {
    match expression {
        Expression::Number { value, .. } => Ok(*value),
        Expression::Symbol { name, span } => symbols.get(name).copied().ok_or_else(|| {
            AssembleError::Parse(ParseError::new(
                format!("Undefined symbol: {}", name),
                span.to_owned(),
            ))
        }),
    }
}

#[derive(Debug, Clone, Copy)]
enum ResolutionState {
    Resolving,
    Resolved(u8),
}

fn collect_symbols(ast: &ProgramAST) -> Result<HashMap<String, u8>, AssembleError> {
    let mut declared_spans: HashMap<String, Vec<Range<usize>>> = HashMap::new();
    let mut first_duplicate = None;
    for line in &ast.lines {
        let label = match line {
            AsmLine::Label { name, span } => Some((name, span)),
            AsmLine::Instruction {
                label: Some(label), ..
            }
            | AsmLine::Directive {
                label: Some(label), ..
            } => Some((&label.name, &label.span)),
            _ => None,
        };
        if let Some((name, span)) = label {
            let spans = declared_spans.entry(name.to_owned()).or_default();
            spans.push(span.to_owned());
            if spans.len() == 2 && first_duplicate.is_none() {
                first_duplicate = Some(name.to_owned());
            }
        }
    }
    if let Some(name) = first_duplicate {
        return Err(AssembleError::DuplicateSymbol {
            definition_spans: declared_spans.remove(&name).unwrap(),
            name,
        });
    }

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

        let label = label.as_ref().ok_or_else(|| {
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
        definitions.insert(label.name.to_owned(), expression);
    }

    let mut symbols = HashMap::new();
    let mut states = HashMap::new();
    let mut memory = Memory::default();

    for line in &ast.lines {
        match line {
            AsmLine::Label { name, span } => {
                define_address(&mut symbols, name, memory.get_pc(), span)?;
            }
            AsmLine::Instruction { label, instr } => {
                if let Some(label) = label {
                    define_address(&mut symbols, &label.name, memory.get_pc(), &label.span)?;
                }
                memory
                    .inc_pc(instr.size())
                    .map_err(|_| AssembleError::OverflowFromInstruction(instr.to_owned()))?;
            }
            AsmLine::Directive { label, dir } if dir.name == Directive::Equ => {}
            AsmLine::Directive { label, dir } => {
                if let Some(label) = label {
                    define_address(&mut symbols, &label.name, memory.get_pc(), &label.span)?;
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
    name: &str,
    address: u8,
    _span: &Range<usize>,
) -> Result<(), AssembleError> {
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

pub fn emit_s19(output: &AssemblyOutput) -> String {
    // Each record holds up to 30 contiguous, explicitly initialized bytes.
    let mut records = Vec::new();
    let mut addr = 0usize;

    while addr < output.memory.len() {
        if !output.initialized[addr] {
            addr += 1;
            continue;
        }

        let start = addr;
        while addr < output.memory.len() && output.initialized[addr] && addr - start < 30 {
            addr += 1;
        }
        records.push(create_s1_record(
            &output.memory,
            start as u8,
            (addr - 1) as u8,
        ));
    }

    // qaflisp uses the first emitted address for the S9 termination record.
    // FLISP's processor entry point remains the reset vector stored at $FF.
    if let Some(start_addr) = output.first_emitted {
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

pub fn emit_fmem(output: &AssemblyOutput, file_name: &str) -> String {
    let mut out = format!("File: {file_name}\n\n # ClearAllMemory\n # ClearAllRegisters");

    for (adr, byte) in output.memory.iter().enumerate() {
        if output.initialized[adr] {
            out.push_str(&format!("\n #setMemory  {:02X}={:02X}", adr, byte))
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::format_addresses;

    #[test]
    fn formats_one_address() {
        assert_eq!(format_addresses(&[0x20]), "address $20");
    }

    #[test]
    fn formats_one_contiguous_range() {
        assert_eq!(format_addresses(&[0x20, 0x21, 0x22]), "addresses $20–$22");
    }

    #[test]
    fn formats_multiple_ranges_and_single_addresses() {
        assert_eq!(
            format_addresses(&[0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x29]),
            "addresses $20–$22, $24–$26, $29"
        );
    }

    #[test]
    fn sorts_and_deduplicates_addresses_before_formatting() {
        assert_eq!(
            format_addresses(&[0x26, 0x20, 0x21, 0x20, 0x25, 0x24]),
            "addresses $20–$21, $24–$26"
        );
    }
}
