# FLISP Assembler

## Instruction encoding snapshots

Instruction encoding cases are defined in
`tests/instruction_encoding_snapshots.rs`. The tests assemble each case and compare the rendered
assembly and machine code with files in `tests/snapshots/`.

A normal test run never replaces an accepted snapshot. If generated output differs, the test fails
and writes the new output beside it as `<snapshot>.new` for inspection.

Run the snapshot tests with:

```sh
cargo test -p assembler --test instruction_encoding_snapshots
```

To create or intentionally replace the accepted snapshots, set
`UPDATE_ENCODING_SNAPSHOTS`:

```sh
UPDATE_ENCODING_SNAPSHOTS=1 \
  cargo test -p assembler --test instruction_encoding_snapshots
```

Always review the resulting snapshot diff before committing it. Initial encoding snapshots should
also be checked against qaflisp and the FLISP handbook.
