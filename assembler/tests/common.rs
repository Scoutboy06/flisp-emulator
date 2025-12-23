use assembler::codegen::{assemble, emit_fmem, emit_s19};
use std::{fs, process::Command};
use tempfile::tempdir;

pub fn make_test(src: &str) {
    let tmpdir = tempdir().unwrap();
    let dir = tmpdir.path();

    let input_path = dir.join("test.sflisp");
    fs::write(&input_path, src).unwrap();

    let output = Command::new("qaflisp")
        .arg("-L")
        .arg(&input_path)
        .current_dir(dir)
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "qaflisp failed to assemble:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    };

    let s19 = fs::read_to_string(dir.join("test.s19")).unwrap();
    let fmem = fs::read_to_string(dir.join("test.fmem")).unwrap();

    let mem = assemble(
        src,
        input_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    )
    .expect("Failed to assemble source code");

    let my_s19 = emit_s19(&mem);
    let my_fmem = emit_fmem(&mem, "test.fmem");

    compare_s19(&s19, &my_s19);
    compare_fmem(&fmem, &my_fmem);
}

fn compare_s19(expected: &str, actual: &str) {
    pretty_assertions::assert_eq!(expected, actual);
}

fn compare_fmem(expected: &str, actual: &str) {
    let expected_without_head = expected.lines().skip(2).collect::<Vec<_>>().join("\n");
    pretty_assertions::assert_eq!(expected_without_head, actual);
}
