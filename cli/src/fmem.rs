use ariadne::{Label, Report, ReportKind, Source};
use std::{io, ops::Range, path::PathBuf};

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

type ParseResult = Result<[u8; 256], ParseError>;

pub fn parse_fmem(path: PathBuf) -> ParseResult {
    let file_str = path.to_string_lossy().to_string();
    let src = std::fs::read_to_string(&path).map_err(|e| ParseError {
        msg: e.to_string(),
        span: 0..0,
        src: String::new(),
        file: file_str.clone(),
    })?;

    let mut mem = [0u8; 256];

    for (line_idx, line) in src.lines().enumerate() {
        let line_start: usize = src
            .split_inclusive('\n')
            .take(line_idx)
            .map(|s| s.len())
            .sum();
        let line_end = line_start + line.len();
        let span = line_start..line_end;

        // parse directive if present
        let rest = match parse_directive(line, span.clone(), &src, &file_str)? {
            None => continue, // ignore non-directive lines
            Some(r) => r,     // only #setMemory lines reach here
        };

        // Now parse <adr>=<val>
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        let base = line_start + indent + "#setMemory ".len();

        let mut parts = rest.split('=');
        let adr = parts
            .next()
            .ok_or(err("expected <adr>=<val>", span.clone(), &src, &file_str))?;
        let val_start = base + adr.len() + 1;
        let val = parts.next().ok_or(err(
            "expected <adr>=<val>",
            val_start..line_end,
            &src,
            &file_str,
        ))?;

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

        let adr = hex_byte(adr.as_bytes())
            .map_err(|_| err("invalid hex digit", base..base + adr.len(), &src, &file_str))?;
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

    Ok(mem)
}

fn parse_directive<'a>(
    line: &'a str,
    span: Range<usize>,
    src: &str,
    file: &str,
) -> Result<Option<&'a str>, ParseError> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return Ok(None);
    }

    // It IS a directive
    const PREFIX: &str = "#setMemory ";
    if trimmed.starts_with(PREFIX) {
        return Ok(Some(&trimmed[PREFIX.len()..]));
    }

    // Unknown directive → error
    let directive_end = match trimmed.find(char::is_whitespace) {
        Some(pos) => pos,
        None => trimmed.len(),
    };
    let dir_start_in_line = line.len() - trimmed.len();
    let dir_start = span.start + dir_start_in_line;
    let dir_end = dir_start + directive_end;

    Err(err("unknown directive", dir_start..dir_end, src, file))
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
