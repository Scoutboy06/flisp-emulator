use std::io;
use std::path::PathBuf;

use crate::parser::Parser;

pub fn run_assemble(input: PathBuf, _output: PathBuf) -> io::Result<()> {
    let file = std::fs::read_to_string(input)?;
    let mut parser = Parser::new(&file);
    let res = parser.parse();

    dbg!(&res);

    Ok(())
}
