use assembler::parser::{AsmLine, Parser};

#[test]
fn parses_label_only_line() {
    let program = Parser::from_source("start:\nNOP\n").parse().unwrap();

    assert!(matches!(
        &program.lines[..],
        [AsmLine::Label { name, .. }, AsmLine::Instruction { .. }] if name == "start"
    ));
}

#[test]
fn rejects_trailing_tokens_on_a_statement() {
    let error = Parser::from_source("ADDA #1 unexpected\n")
        .parse()
        .unwrap_err();

    assert_eq!(error.msg, "Expected end of line");
}

#[test]
fn accepts_empty_lines() {
    let program = Parser::from_source("\n\nNOP\n\n").parse().unwrap();

    assert_eq!(program.lines.len(), 1);
}
