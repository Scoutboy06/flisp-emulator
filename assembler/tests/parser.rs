use assembler::{
    codegen::{AssembleError, assemble},
    parser::{AsmLine, Expression, Operand, Parser},
};

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
fn preserves_symbol_reference_spans() {
    let program = Parser::from_source("BRA target\n").parse().unwrap();
    let AsmLine::Instruction { instr, .. } = &program.lines[0] else {
        panic!("expected instruction");
    };

    assert!(matches!(
        &instr.operands[..],
        [Operand::RelAdr(Expression::Symbol { name, span })]
            if name == "target" && span == &(4..10)
    ));
}

#[test]
fn undefined_symbol_error_uses_reference_span() {
    let error = assemble("BRA target\n", "test.sflisp".to_owned()).unwrap_err();
    let AssembleError::Parse(error) = error else {
        panic!("expected parse error");
    };

    assert_eq!(error.msg, "Undefined symbol: target");
    assert_eq!(error.span, 4..10);
}

#[test]
fn equ_defines_a_constant_without_emitting_bytes() {
    let memory = assemble(
        "VALUE EQU $2A\nALIAS EQU VALUE\nORG $20\nLDA #ALIAS\n",
        "test.sflisp".to_owned(),
    )
    .unwrap();

    assert_eq!(memory[0x20], 0xf0);
    assert_eq!(memory[0x21], 0x2a);
    assert_eq!(memory[0], 0);
}

#[test]
fn equ_requires_a_symbol_definition() {
    let error = assemble("EQU $2A\n", "test.sflisp".to_owned()).unwrap_err();
    let AssembleError::Parse(error) = error else {
        panic!("expected parse error");
    };

    assert_eq!(error.msg, "EQU directives require a symbol definition");
}

#[test]
fn accepts_empty_lines() {
    let program = Parser::from_source("\n\nNOP\n\n").parse().unwrap();

    assert_eq!(program.lines.len(), 1);
}
