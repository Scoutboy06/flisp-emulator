use ariadne::Source;
use assembler::{
    codegen::{AssembleError, DependencyEdge, assemble, emit_fmem, emit_s19},
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

    assert_eq!(memory.memory()[0x20], 0xf0);
    assert_eq!(memory.memory()[0x21], 0x2a);
    assert_eq!(memory.memory()[0], 0);
}

#[test]
fn equ_supports_forward_aliases() {
    let memory = assemble(
        "FIRST EQU SECOND\nSECOND EQU $2A\nORG $20\nLDA #FIRST\n",
        "test.sflisp".to_owned(),
    )
    .unwrap();

    assert_eq!(&memory.memory()[0x20..=0x21], &[0xf0, 0x2a]);
}

#[test]
fn reports_and_visualizes_circular_equ_definitions() {
    let source = "FIRST EQU SECOND\nSECOND EQU THIRD\nTHIRD EQU FIRST\n";
    let error = assemble(source, "test.sflisp".to_owned()).unwrap_err();
    let AssembleError::CircularDefinition { edges } = &error else {
        panic!("expected circular definition error, got {error:?}");
    };

    assert_eq!(
        edges,
        &[
            DependencyEdge {
                from: "FIRST".to_owned(),
                to: "SECOND".to_owned(),
                reference_span: 10..16,
            },
            DependencyEdge {
                from: "SECOND".to_owned(),
                to: "THIRD".to_owned(),
                reference_span: 28..33,
            },
            DependencyEdge {
                from: "THIRD".to_owned(),
                to: "FIRST".to_owned(),
                reference_span: 44..49,
            },
        ]
    );

    let mut rendered = Vec::new();
    error
        .build_report("test.sflisp")
        .write(("test.sflisp", Source::from(source)), &mut rendered)
        .unwrap();
    let rendered = String::from_utf8(rendered).unwrap();
    assert!(rendered.contains("Circular symbol definition"));
    assert!(rendered.contains("FIRST depends on SECOND"));
    assert!(rendered.contains("SECOND depends on THIRD"));
    assert!(rendered.contains("THIRD depends on FIRST, completing the cycle"));
    assert!(rendered.contains("dependency cycle: FIRST -> SECOND -> THIRD -> FIRST"));
}

#[test]
fn branches_encode_targets_relative_to_the_next_instruction() {
    let memory = assemble(
        "ORG $20\nstart: BRA forward\nNOP\nforward: BRA start\nBRA $30\nJMP $30\n",
        "test.sflisp".to_owned(),
    )
    .unwrap();

    assert_eq!(
        &memory.memory()[0x20..=0x28],
        &[
            0x21, 0x01, // Forward: $23 - $22
            0x00, // NOP
            0x21, 0xfb, // Backward: $20 - $25, modulo 256
            0x21, 0x09, // Numeric target: $30 - $27
            0x33, 0x30, // JMP remains an absolute address
        ]
    );
}

#[test]
fn branch_aliases_use_their_canonical_opcodes() {
    let memory = assemble(
        "ORG $20\nBHS target\nBLO target\ntarget: NOP\n",
        "test.sflisp".to_owned(),
    )
    .unwrap();

    assert_eq!(
        &memory.memory()[0x20..=0x24],
        &[
            0x29, 0x02, // BHS is BCC
            0x28, 0x00, // BLO is BCS
            0x00,
        ]
    );
}

#[test]
fn output_preserves_explicitly_initialized_zeroes() {
    let output = assemble(
        "ORG $20\nFCB $AA\nORG $30\nFCB $00,$00,$BB\n",
        "test.sflisp".to_owned(),
    )
    .unwrap();

    assert!(output.initialized()[0x20]);
    assert!(!output.initialized()[0x21]);
    assert_eq!(&output.memory()[0x30..=0x32], &[0x00, 0x00, 0xbb]);
    assert!(output.initialized()[0x30..=0x32].iter().all(|value| *value));

    let s19 = emit_s19(&output);
    assert!(s19.lines().any(|line| line.starts_with("S10600300000BB")));

    let fmem = emit_fmem(&output, "test.fmem");
    assert!(fmem.contains("#setMemory  30=00"));
    assert!(fmem.contains("#setMemory  31=00"));
}

#[test]
fn s19_uses_the_first_emitted_address_for_its_start_record() {
    let output = assemble(
        "ORG $00\nFCB $AA\nORG $20\nNOP\nORG $FF\nFCB $20\n",
        "test.sflisp".to_owned(),
    )
    .unwrap();

    assert_eq!(emit_s19(&output).lines().last(), Some("S9030000FC"));
    assert_eq!(output.memory()[0xff], 0x20);
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
