use std::{fs::File, path::PathBuf};

use assembler::codegen::assemble;
use clap::{Parser, Subcommand, builder::OsStr};
use emulator::Emulator;
use tui::ui::EmulatorVisualizer;

use crate::fmem::parse_fmem;
use flisp_core::{fmem, s19::parse_s19};

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

    let extension = input.extension();

    let mem: [u8; 256] = if input.extension() == Some(&OsStr::from("s19")) {
        match parse_s19(input) {
            Ok(mem) => mem,
            Err(e) => {
                panic!("S19 Parse Error: {:?}", e);
            }
        }
    } else if extension == Some(&OsStr::from("fmem")) {
        match parse_fmem(input) {
            Ok(fmem) => fmem.mem,
            Err(e) => {
                e.report();
                std::process::exit(1);
            }
        }
    } else if extension == Some(&OsStr::from("sflisp")) {
        todo!()
    } else {
        panic!("Unsupported file extension");
    };

    let mut program = Emulator::default();
    program.load_memory(&mem);
    EmulatorVisualizer::viz(&mut program).unwrap()
}
