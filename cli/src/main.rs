#![allow(unused)]
use std::{fs::File, path::PathBuf};

use assembler::run_assemble;
use clap::{CommandFactory, Parser, Subcommand, builder::OsStr};
use emulator::Emulator;
use tui::ui::EmulatorVisualizer;

use crate::fmem::parse_fmem;

mod fmem;

/*
CLI usage:
- TODO: `myprogram <file>.fmem` - visualize
- TODO: `myprogram assemble <file>.asm [-o <output>.fmem]` - assemble
*/

#[derive(Parser, Debug)]
#[command(name = "Flisp Emulator", version, about = "Unified tool for the Flisp Emulator", long_about = None)]
struct Cli {
    #[arg(global = true)]
    fmem_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Assemble an assembly file into a .fmem file")]
    Assemble {
        #[arg(short, long, default_value = "output.fmem")]
        output: PathBuf,
        input: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Assemble { input, output }) => {
            run_assemble(input, output)?;
        }
        None => {
            if let Some(fmem) = args.fmem_file {
                run_visualize(fmem);
            } else {
                Cli::command().print_help()?;
                println!();
            }
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
    program.load_memory(&fmem);
    EmulatorVisualizer::viz(&mut program).unwrap()
}
