use ariadne::{Label, Report, ReportKind, Source};
use std::{ops::Range, path::PathBuf};

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub span: Range<usize>,
    pub src: String,
    pub file: String,
}

impl ParseError {
    pub fn report(&self) {
        Report::build(ReportKind::Error, (&self.file[..], self.span.clone()))
            .with_message(&self.msg)
            .with_label(Label::new((&self.file[..], self.span.clone())).with_message("here"))
            .finish()
            .print((&self.file[..], Source::from(&self.src)))
            .unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct FmemParse {
    pub mem: [u8; 256],
    pub clear_all_memory: bool,
    pub clear_all_registers: bool,
}

enum Directive<'a> {
    SetMemory(&'a str),
    ClearAllMemory,
    ClearAllRegisters,
}

pub fn parse_fmem(path: PathBuf) -> Result<FmemParse, ParseError> {
    let file_str = path.to_string_lossy().to_string();
    let src = std::fs::read_to_string(&path).map_err(|e| ParseError {
        msg: e.to_string(),
        span: 0..0,
        src: String::new(),
        file: file_str.clone(),
    })?;

    let mut mem = [0_u8; 256];
    let mut clear_all_memory = false;
    let mut clear_all_registers = false;

    for (line_idx, line) in src.lines().enumerate() {
        let line_start: usize = src
            .split_inclusive('\n')
            .take(line_idx)
            .map(|s| s.len())
            .sum();
        let line_end = line_start + line.len();
        let span = line_start..line_end;

        match parse_directive(line, span.clone(), &src, &file_str)? {
            None => continue,
            Some(Directive::ClearAllMemory) => {
                clear_all_memory = true;
                continue;
            }
            Some(Directive::ClearAllRegisters) => {
                clear_all_registers = true;
                continue;
            }
            Some(Directive::SetMemory(rest)) => {
                // Now parse <adr>=<val> from `rest`, which starts after the directive
                let trimmed_rest = rest.trim_start();
                // compute base position for the rest in the source
                let leading_ws = rest.len() - trimmed_rest.len();
                let base = line_start + (line.len() - line.trim_start().len()) + 1 + leading_ws;
                // explanation: (line.len()-line.trim_start().len()) is offset to '#', +1 for '#', then leading_ws

                let mut parts = trimmed_rest.split('=');
                let adr = parts
                    .next()
                    .ok_or(err("expected <adr>=<val>", span.clone(), &src, &file_str))?
                    .trim();
                let val_start = base + adr.len() + 1;
                let val = parts
                    .next()
                    .ok_or(err(
                        "expected <adr>=<val>",
                        val_start..line_end,
                        &src,
                        &file_str,
                    ))?
                    .trim();

                if adr.len() != 2 {
                    return Err(err(
                        "address must be exactly two hex digits",
                        base..base + adr.len(),
                        &src,
                        &file_str,
                    ));
                }
                if val.len() != 2 {
                    return Err(err(
                        "value must be exactly two hex digits",
                        val_start..val_start + val.len(),
                        &src,
                        &file_str,
                    ));
                }

                let adr = hex_byte(adr.as_bytes()).map_err(|_| {
                    err("invalid hex digit", base..base + adr.len(), &src, &file_str)
                })?;
                let val = hex_byte(val.as_bytes()).map_err(|_| {
                    err(
                        "invalid hex digit",
                        val_start..val_start + val.len(),
                        &src,
                        &file_str,
                    )
                })?;

                mem[adr as usize] = val;
            }
        }
    }

    Ok(FmemParse {
        mem,
        clear_all_memory,
        clear_all_registers,
    })
}

fn parse_directive<'a>(
    line: &'a str,
    span: Range<usize>,
    src: &str,
    file: &str,
) -> Result<Option<Directive<'a>>, ParseError> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return Ok(None);
    }

    // Skip '#' and any whitespace after it
    let after_hash = &trimmed[1..].trim_start();

    // extract directive name (token up to whitespace or end)
    let directive_name_end = after_hash
        .find(char::is_whitespace)
        .unwrap_or(after_hash.len());
    let directive_name = &after_hash[..directive_name_end];

    // To compute the span of the directive name within the whole source, find its offset
    let dir_start_in_line = line.len() - trimmed.len(); // index of '#'
    // position of directive_name start: after '#' + whitespace after '#'
    let whitespace_after_hash_len = trimmed[1..].len() - after_hash.len();
    let name_start_in_line = dir_start_in_line + 1 + whitespace_after_hash_len;
    let dir_start = span.start + name_start_in_line;
    let dir_end = dir_start + directive_name_end;

    match directive_name {
        "setMemory" => {
            // the rest after the directive name
            let rest = &after_hash[directive_name_end..];
            Ok(Some(Directive::SetMemory(rest)))
        }
        "ClearAllMemory" => Ok(Some(Directive::ClearAllMemory)),
        "ClearAllRegisters" => Ok(Some(Directive::ClearAllRegisters)),
        _ => Err(err("unknown directive", dir_start..dir_end, src, file)),
    }
}

fn err(msg: &str, span: Range<usize>, src: &str, file: &str) -> ParseError {
    ParseError {
        msg: msg.to_string(),
        span,
        src: src.to_string(),
        file: file.to_string(),
    }
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn hex_byte(two: &[u8]) -> Result<u8, ()> {
    if two.len() != 2 {
        return Err(());
    }
    let hi = hex_digit(two[0]).ok_or(())?;
    let lo = hex_digit(two[1]).ok_or(())?;
    Ok((hi << 4) | lo)
}
