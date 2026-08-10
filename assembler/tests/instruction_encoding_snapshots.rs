use std::{fs, path::Path};

use assembler::codegen::assemble;

const ORIGIN: usize = 0x20;
const INHERENT: &[&str] = &[
    "NOP", "CLRA", "NEGA", "INCA", "DECA", "TSTA", "COMA", "LSLA", "LSRA", "ROLA", "RORA", "ASRA",
    "PSHA", "PSHX", "PSHY", "PSHC", "PULA", "PULX", "PULY", "PULC", "RTS", "RTI",
];

#[test]
fn inherent_instruction_encodings() {
    assert_encoding_snapshot("inherent", INHERENT);
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
