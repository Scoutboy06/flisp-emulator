use std::fs;
use std::path::{Path, PathBuf};

use assembler::codegen::{assemble, emit_fmem, emit_s19};
use assembler::parser::ParseError;

#[test]
fn flisp_matches_qaflisp() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden_files");

    let cases = discover_cases(&root);
    assert!(!cases.is_empty(), "No test cases found");

    let mut failures = Vec::new();

    for case in cases {
        if let Err(err) = run_case(&case) {
            failures.push(err);
        }
    }

    if !failures.is_empty() {
        panic!(
            "Compatibility failures ({} cases):\n\n",
            failures.len(),
            // failures.join("\n\n")
        );
    }
}

#[derive(Debug)]
enum TestError {
    IO(String),
    Parse(ParseError),
    S19Mismatch { expected: String, actual: String },
    FmemMismatch { expected: String, actual: String },
}

fn discover_cases(root: &Path) -> Vec<PathBuf> {
    fs::read_dir(root)
        .unwrap()
        .filter_map(|e| {
            let dir = e.unwrap().path();
            let name = dir.file_name()?.to_str()?;
            let sflisp = dir.join(format!("{name}.sflisp"));

            if sflisp.exists() { Some(dir) } else { None }
        })
        .collect()
}

fn run_case(case_dir: &Path) -> Result<(), TestError> {
    let name = case_dir
        .file_name()
        .ok_or_else(|| TestError::IO("Case directory has no name".into()))?
        .to_string_lossy()
        .to_string();

    // Define file paths
    let source_path = case_dir.join(format!("{name}.sflisp"));
    let s19_path = case_dir.join(format!("{name}.s19"));
    let fmem_path = case_dir.join(format!("{name}.fmem"));

    // Check that all files exist before proceeding
    for (path, desc) in &[
        (&source_path, "source"),
        (&s19_path, "expected s19"),
        (&fmem_path, "expected fmem"),
    ] {
        if !path.exists() {
            return Err(TestError::IO(format!(
                "{}: missing {} file: {}",
                name,
                desc,
                path.display()
            )));
        }
    }

    // Read input files
    let source = fs::read_to_string(&source_path)
        .map_err(|e| TestError::IO(format!("{name}: failed to read source: {e}")))?;
    let s19_expected = fs::read_to_string(&s19_path)
        .map_err(|e| TestError::IO(format!("{name}: failed to read expected s19: {e}")))?;
    let fmem_expected = fs::read_to_string(&fmem_path)
        .map_err(|e| TestError::IO(format!("{name}: failed to read expected fmem: {e}")))?;

    // Assemble the program
    let program = match assemble(&source, name.clone()) {
        Ok(Ok(ast)) => ast,
        Ok(Err(parse_err)) => return Err(TestError::Parse(parse_err)),
        Err(io_err) => {
            return Err(TestError::IO(format!(
                "{name}: assembly IO error: {io_err}"
            )));
        }
    };

    // Emit outputs
    let our_s19_output = emit_s19(&program);
    let our_fmem_output = emit_fmem(&program);

    // Compare outputs
    let s19_match = compare_s19(&s19_expected, &our_s19_output);
    if !s19_match {
        return Err(TestError::S19Mismatch {
            expected: s19_expected,
            actual: our_s19_output,
        });
    };

    let fmem_match = compare_fmem(&fmem_expected, &our_fmem_output);
    if !fmem_match {
        return Err(TestError::FmemMismatch {
            expected: fmem_expected,
            actual: our_fmem_output,
        });
    }

    Ok(())
}

fn compare_s19(expected: &str, actual: &str) -> bool {
    expected.trim() != actual.trim()
}

fn compare_fmem(expected: &str, actual: &str) -> bool {
    expected.trim() != actual.trim()
}
