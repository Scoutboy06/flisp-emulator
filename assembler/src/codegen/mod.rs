use std::collections::HashMap;

use crate::{
    lexer::directive::Directive,
    parser::{AsmInstruction, AsmLine, Atom, Operand, ParseError, Parser, ProgramAST},
};

#[derive(Debug)]
pub enum AssembleError {
    Parse(ParseError),
    Overflow(AsmInstruction),
}

pub fn assemble(src: &str, file_path: String) -> Result<[u8; 256], AssembleError> {
    let ast = Parser::from_source(src)
        .with_source_name(file_path)
        .parse()
        .map_err(AssembleError::Parse)?;

    let symbols = collect_symbols(&ast)?;
    dbg!(&symbols);

    let mut memory = [0u8; 256];
    let mut pc: u8 = 0;

    for line in ast.lines {
        match line {
            AsmLine::Instruction(ins) => {
                memory[pc as usize] = ins.opcode;
                let (new_pc, overflow1) = pc.overflowing_add(1);
                let (end_pc, overflow2) = new_pc.overflowing_add(ins.operands.len() as u8);
                if overflow1 || overflow2 {
                    return Err(AssembleError::Overflow(ins));
                }
                for (i, operand) in ins.operands.iter().enumerate() {
                    match operand {
                        Operand::Imm(val) => memory[new_pc as usize + i] = *val,
                        Operand::AbsAdr(addr) => memory[new_pc as usize + i] = *addr,
                        Operand::RelAdr(offset) => memory[new_pc as usize + i] = *offset,
                        Operand::N(n) => memory[new_pc as usize + i] = *n,
                        Operand::Reg(_) => { /* Not to be included in binary */ }
                    };
                }
                pc = end_pc;
            }
            AsmLine::Directive(dir) => match dir.name {
                Directive::Org => match dir.args.first() {
                    Some(Atom::Number(n)) => {
                        pc = *n;
                    }
                    Some(Atom::Symbol(sym)) => {
                        let new_addr = symbols.get(sym).ok_or_else(|| {
                            AssembleError::Parse(ParseError::new(
                                format!("Undefined symbol: {}", sym),
                                dir.span,
                            ))
                        })?;
                        pc = *new_addr;
                    }
                    _ => {
                        return Err(AssembleError::Parse(ParseError::new(
                            "ORG directive requires an address argument".to_string(),
                            dir.span,
                        )));
                    }
                },
                Directive::Equ => todo!(),
                Directive::Fcb => todo!(),
                Directive::Fcs => todo!(),
                Directive::Rmb => todo!(),
            },
            AsmLine::Symbol(_) => { /* We have already collected symbols. Do nothing. */ }
        }
    }

    Ok(memory)
}

fn collect_symbols(ast: &ProgramAST) -> Result<HashMap<String, u8>, AssembleError> {
    let mut symbols: HashMap<String, u8> = HashMap::new();

    let mut pc: u8 = 0;
    for line in &ast.lines {
        match line {
            AsmLine::Symbol(sym) => {
                if symbols.contains_key(&sym.name) {
                    return Err(AssembleError::Parse(ParseError::new(
                        format!("Duplicate symbol: {}", sym.name),
                        sym.span.clone(),
                    )));
                }
                symbols.insert(sym.name.clone(), pc);
            }
            AsmLine::Directive(dir) => match dir.name {
                Directive::Org => match dir.args.first() {
                    Some(Atom::Number(n)) => {
                        pc = *n;
                    }
                    Some(Atom::Symbol(sym)) => {
                        let new_addr = symbols.get(sym).ok_or_else(|| {
                            AssembleError::Parse(ParseError::new(
                                format!("Undefined symbol: {}", sym),
                                dir.span.clone(),
                            ))
                        })?;
                        pc = *new_addr;
                    }
                    _ => {
                        return Err(AssembleError::Parse(ParseError::new(
                            "ORG directive requires an address argument".to_string(),
                            dir.span.clone(),
                        )));
                    }
                },
                Directive::Equ => todo!(),
                Directive::Fcb => todo!(),
                Directive::Fcs => todo!(),
                Directive::Rmb => todo!(),
            },
            AsmLine::Instruction(ins) => {
                let (new_pc, overflow) = pc.overflowing_add(ins.size());
                if overflow {
                    return Err(AssembleError::Overflow(ins.clone()));
                }
                pc = new_pc;
            }
        }
    }

    Ok(symbols)
}

pub fn emit_s19(_program: &ProgramAST) -> String {
    todo!()
}

pub fn emit_fmem(_program: &ProgramAST) -> String {
    todo!()
}
