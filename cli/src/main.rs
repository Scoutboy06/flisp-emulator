use std::{fs::File, path::PathBuf};

use assembler::codegen::{assemble, emit_fmem};
use clap::{CommandFactory, Parser, Subcommand, builder::OsStr};
use emulator::Emulator;
use tui::ui::EmulatorVisualizer;

use crate::fmem::parse_fmem;
use flisp_core::fmem;

/*
CLI usage:
- TODO: `myprogram <file>.fmem` - visualize
- TODO: `myprogram assemble <file>.asm [-o <output>.fmem]` - assemble
*/

#[derive(Parser, Debug)]
#[command(name = "flisp", version, about = "Unified tool for the Flisp Emulator", long_about = None)]
enum Cli {
    #[command(about = "Run and visualize a flisp program. Supports .sflisp, .fmem and .s19 files")]
    Run { input: PathBuf },
    #[command(about = "Assemble your source code. Supports .sflisp files")]
    Assemble {
        #[arg(short, long)]
        output: PathBuf,
        input: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum Commands {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args {
        Cli::Run { input } => {
            run_visualize(input);
        }
        Cli::Assemble { input, output: _ } => {
            let file = std::fs::read_to_string(input.to_string_lossy().to_string())?;
            let file_path = input.to_string_lossy().to_string();
            let _res = assemble(&file, file_path);
            dbg!(&_res);
            println!("Assemble completed successfully.");
        }
    }

    Ok(())
}

fn run_visualize(input: PathBuf) {
    let mut _file = File::open(input.clone()).expect("Failed to open file");

    if input.extension() != Some(&OsStr::from("fmem")) {
        panic!("Invalid file type");
    }

    let fmem = match parse_fmem(input) {
        Ok(fmem) => fmem,
        Err(e) => {
            e.report();
            std::process::exit(1);
        }
    };

    let mut program = Emulator::default();
    program.load_memory(&fmem.mem);
    EmulatorVisualizer::viz(&mut program).unwrap()
}
