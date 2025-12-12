use std::io;
use std::path::PathBuf;

use crate::lexer::Lexer;
use crate::lexer::token::TokenKind;

pub fn run_assemble(input: PathBuf, _output: PathBuf) -> io::Result<()> {
    let file = std::fs::read_to_string(input)?;
    let mut lex = Lexer::new(&file);

    loop {
        let tok = lex.next_token();
        dbg!(&tok);
        if tok.kind == TokenKind::Eof {
            break;
        }
    }

    Ok(())
}
