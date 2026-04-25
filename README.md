# FLISP Emulator Project

> **Disclaimer:** This is a work-in-progress project. Features, usage, and code structure may change rapidly. Use with care and check for updates frequently.

## Overview

FLISP Emulator is a modular, Rust-based emulator and assembler workspace, targeting the FLISP architecture and tools. It is organized as a Rust multi-crate workspace with several focused sub-projects:

- **assembler**: Implements assembly source parsing, code generation, and related utilities.
- **emulator**: An internal library for simulating FLISP instructions, registers, and memory.
- **flisp-core**: Provides foundational types and utilities for FLISP emulation and assembly.
- **cli**: Command-line interface for interacting with the emulator and assembler.
- **tui**: Terminal UI for a richer emulation experience.

## Key Features

- FLISP assembler and emulator in pure Rust
- Extensible modular design via workspace crates
- CLI and TUI interfaces for user workflows
- Advanced editing, breakpoints, and state management

---

## Installation (Users)

**Prerequisites**

- Rust 2024 edition, nightly channel recommended
- Supported on major OSes (Linux, macOS, Windows)

**Install with Cargo:**

```sh
cargo install --path .
```

Or build all workspace binaries:

```sh
cargo build --release
```

**Quick Start:**
Run the CLI binary:

```sh
cargo run -p cli -- --help
```

---

## Usage Example

Example CLI usage:

```sh
# Assemble a FLISP source file (.sflisp) to .s19 or .fmem
flisp-cli assemble program.sflisp

# Run and visualize a FLISP program. Input can be .sflisp, .s19 or .fmem
flisp-cli run program.sflisp
flisp-cli run compiled.s19
flisp-cli run memory_image.fmem
```

GUI users can launch the TUI:

```sh
cargo run -p tui
```

---

## Sub-crate Summary

- **assembler/**: Assemblies FLISP source, provides parsing and codegen utilities.
- **emulator/**: Simulates system instructions, memory visualization, debugging, breakpoints.
- **flisp-core/**: Central logic and types shared across crates.
- **cli/**: Command-line interface for emulation and assembly tasks.
- **tui/**: Terminal UI, keybindings, and interactive features.

---

## Development Setup

**Clone and Build:**

```sh
git clone <repo-url>
cd flisp-emulator
cargo build
```

Development uses the nightly Rust toolchain. See `rust-toolchain.toml` for details.

---

## Testing

Run all tests in the workspace:

```sh
cargo test --workspace
```

**Assembler Sub-crate Testing:**
To run assembler tests, you need the `qaflisp` assembler. This is often bundled with `digiflisp`. Download it from:
[https://gbgmv.se/studies.html](https://gbgmv.se/studies.html)

---

## License

See [LICENSE](LICENSE) for details.
