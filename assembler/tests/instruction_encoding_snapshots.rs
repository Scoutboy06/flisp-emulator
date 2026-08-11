use std::{fs, path::Path};

use assembler::codegen::assemble;

const ORIGIN: usize = 0x20;
const INHERENT: &[&str] = &[
    "NOP", "CLRA", "NEGA", "INCA", "DECA", "TSTA", "COMA", "LSLA", "LSRA", "ROLA", "RORA", "ASRA",
    "PSHA", "PSHX", "PSHY", "PSHC", "PULA", "PULX", "PULY", "PULC", "RTS", "RTI",
];

const IMMEDIATE: &[&str] = &[
    "ANDCC #$42",
    "ORCC #$42",
    "LDX #$42",
    "LDY #$42",
    "LDSP #$42",
    "SBCA #$42",
    "SUBA #$42",
    "ADCA #$42",
    "ADDA #$42",
    "CMPA #$42",
    "BITA #$42",
    "ANDA #$42",
    "ORA #$42",
    "EORA #$42",
    "CMPX #$42",
    "CMPY #$42",
    "CMPSP #$42",
    "LDA #$42",
];

const RELATIVE: &[&str] = &[
    "BSR $42", "BRA $42", "BMI $42", "BPL $42", "BEQ $42", "BNE $42", "BVS $42", "BVC $42",
    "BCS $42", "BLO $42", "BCC $42", "BHS $42", "BHI $42", "BLS $42", "BGT $42", "BGE $42",
    "BLE $42", "BLT $42",
];

const INDEXED: &[&str] = &[
    "STX $42,SP",
    "STY $42,SP",
    "STSP $42,SP",
    "CLR $42,SP",
    "NEG $42,SP",
    "INC $42,SP",
    "DEC $42,SP",
    "TST $42,SP",
    "COM $42,SP",
    "LSL $42,SP",
    "LSR $42,SP",
    "ROL $42,SP",
    "ROR $42,SP",
    "ASR $42,SP",
    "STX $42,X",
    "STY $42,X",
    "STSP $42,X",
    "JMP $42,X",
    "JSR $42,X",
    "CLR $42,X",
    "NEG $42,X",
    "INC $42,X",
    "DEC $42,X",
    "TST $42,X",
    "COM $42,X",
    "LSL $42,X",
    "LSR $42,X",
    "ROL $42,X",
    "ROR $42,X",
    "ASR $42,X",
    "STX $42,Y",
    "STY $42,Y",
    "STSP $42,Y",
    "JMP $42,Y",
    "JSR $42,Y",
    "CLR $42,Y",
    "NEG $42,Y",
    "INC $42,Y",
    "DEC $42,Y",
    "TST $42,Y",
    "COM $42,Y",
    "LSL $42,Y",
    "LSR $42,Y",
    "ROL $42,Y",
    "ROR $42,Y",
    "ASR $42,Y",
    "LDX $42,SP",
    "LDY $42,SP",
    "LDSP $42,SP",
    "SBCA $42,SP",
    "SUBA $42,SP",
    "ADCA $42,SP",
    "ADDA $42,SP",
    "CMPA $42,SP",
    "BITA $42,SP",
    "ANDA $42,SP",
    "ORA $42,SP",
    "EORA $42,SP",
    "CMPX $42,SP",
    "CMPY $42,SP",
    "LEASP $42,SP",
    "LDX $42,X",
    "LDY $42,X",
    "LDSP $42,X",
    "SBCA $42,X",
    "SUBA $42,X",
    "ADCA $42,X",
    "ADDA $42,X",
    "CMPA $42,X",
    "BITA $42,X",
    "ANDA $42,X",
    "ORA $42,X",
    "EORA $42,X",
    "LEAX $42,X",
    "LEAY $42,Y",
    "LEASP $42,X",
    "LDX $42,Y",
    "LDY $42,Y",
    "LDSP $42,Y",
    "SBCA $42,Y",
    "SUBA $42,Y",
    "ADCA $42,Y",
    "ADDA $42,Y",
    "CMPA $42,Y",
    "BITA $42,Y",
    "ANDA $42,Y",
    "ORA $42,Y",
    "EORA $42,Y",
    "LEAX $42,SP",
    "LEAY $42,SP",
    "LEASP $42,Y",
    "STA $42,SP",
    "STA $42,X",
    "STA $42,Y",
    "LDA $42,SP",
    "LDA $42,X",
    "LDA $42,Y",
    "STX A,X",
    "STY A,X",
    "STSP A,X",
    "JMP A,X",
    "JSR A,X",
    "CLR A,X",
    "NEG A,X",
    "INC A,X",
    "DEC A,X",
    "TST A,X",
    "COM A,X",
    "LSL A,X",
    "LSR A,X",
    "ROL A,X",
    "ROR A,X",
    "ASR A,X",
    "STX A,Y",
    "STY A,Y",
    "STSP A,Y",
    "JMP A,Y",
    "JSR A,Y",
    "CLR A,Y",
    "NEG A,Y",
    "INC A,Y",
    "DEC A,Y",
    "TST A,Y",
    "COM A,Y",
    "LSL A,Y",
    "LSR A,Y",
    "ROL A,Y",
    "ROR A,Y",
    "ASR A,Y",
    "STA A,X",
    "STA A,Y",
    "LDA A,X",
    "LDA A,Y",
];

const ABSOLUTE: &[&str] = &[
    "STX $42",
    "STY $42",
    "STSP $42",
    "JMP $42",
    "JSR $42",
    "CLR $42",
    "NEG $42",
    "INC $42",
    "DEC $42",
    "TST $42",
    "COM $42",
    "LSL $42",
    "LSR $42",
    "ROL $42",
    "ROR $42",
    "ASR $42",
    "LDX $42",
    "LDY $42",
    "LDSP $42",
    "SBCA $42",
    "SUBA $42",
    "ADCA $42",
    "ADDA $42",
    "CMPA $42",
    "BITA $42",
    "ANDA $42",
    "ORA $42",
    "EORA $42",
    "CMPX $42",
    "CMPY $42",
    "CMPSP $42",
    "STA $42",
    "LDA $42",
];

#[test]
fn inherent_instruction_encodings() {
    assert_encoding_snapshot("inherent", INHERENT);
}

#[test]
fn immediate_instruction_encodings() {
    assert_encoding_snapshot("immediate", IMMEDIATE);
}

#[test]
fn absolute_instruction_encodings() {
    assert_encoding_snapshot("absolute", ABSOLUTE);
}

#[test]
fn indexed_instruction_encodings() {
    assert_encoding_snapshot("indexed", INDEXED);
}

#[test]
fn relative_instruction_encodings() {
    assert_encoding_snapshot("relative", RELATIVE);
}

fn assert_encoding_snapshot(name: &str, cases: &[&str]) {
    let actual = render_encodings(cases);
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.encoding.sflisp"));

    if std::env::var_os("UPDATE_ENCODING_SNAPSHOTS").is_some() {
        fs::write(&snapshot_path, actual).unwrap();
        return;
    }

    let expected = fs::read_to_string(&snapshot_path).unwrap_or_else(|_| {
        panic!(
            "missing encoding snapshot {}; regenerate it with \
             `UPDATE_ENCODING_SNAPSHOTS=1 cargo test -p assembler --test instruction_encoding_snapshots`",
            snapshot_path.display()
        )
    });
    if actual != expected {
        let pending_path = snapshot_path.with_extension("sflisp.new");
        fs::write(&pending_path, &actual).unwrap();
        panic!(
            "encoding snapshot `{name}` changed; compare {} with {}",
            snapshot_path.display(),
            pending_path.display()
        );
    }
}

fn render_encodings(cases: &[&str]) -> String {
    let width = cases.iter().map(|case| case.len()).max().unwrap_or(0) + 4;
    let mut snapshot = String::new();

    for source in cases {
        let assembly = format!("ORG ${ORIGIN:02X}\n{source}\n");
        let output = assemble(&assembly, format!("{source}.sflisp"))
            .unwrap_or_else(|error| panic!("failed to assemble `{source}`: {error:?}"));
        let bytes = output.initialized()[ORIGIN..]
            .iter()
            .take_while(|initialized| **initialized)
            .enumerate()
            .map(|(offset, _)| format!("{:02X}", output.memory()[ORIGIN + offset]))
            .collect::<Vec<_>>()
            .join(" ");
        snapshot.push_str(&format!("{source:<width$}; {bytes}\n"));
    }

    snapshot
}
