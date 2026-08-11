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
