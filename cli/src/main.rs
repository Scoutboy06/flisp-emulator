use std::{fs::File, path::PathBuf, process::ExitCode};

use assembler::codegen::{assemble, emit_fmem, emit_s19};
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
    Assemble { input: PathBuf },
}

#[derive(Subcommand, Debug)]
enum Commands {}

fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args {
        Cli::Run { input } => {
            run_visualize(input);
        }
        Cli::Assemble { input } => {
            let file = std::fs::read_to_string(input.to_string_lossy().to_string())?;
            let file_stem = input
                .file_stem()
                .expect("Failed to get file stem")
                .to_string_lossy();
            let file_path = input.to_string_lossy().to_string();
            let mem = match assemble(&file, file_path.to_owned()) {
                Ok(mem) => mem,
                Err(error) => {
                    eprintln!("Assemble failed:");
                    error.report_on(&file_path, &file);
                    return Ok(ExitCode::FAILURE);
                }
            };

            for warning in mem.warnings() {
                warning.report_on(&file_path, &file);
            }

            let s19_str = emit_s19(&mem);
            let s19_file_name = format!("{}.s19", file_stem);
            std::fs::write(&s19_file_name, s19_str)?;

            let fmem_file_name = format!("{}.fmem", file_stem);
            let fmem_str = emit_fmem(&mem, &fmem_file_name);
            std::fs::write(&fmem_file_name, fmem_str)?;

            println!("Assemble completed successfully.");
        }
    }

    Ok(ExitCode::SUCCESS)
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
    } else if extension.is_some() {
        panic!(
            "Unsupported file extension: {}",
            extension.unwrap().to_string_lossy()
        );
    } else {
        panic!("Input file has no extension");
    };

    let mut program = Emulator::default();
    program.load_memory(&mem);
    EmulatorVisualizer::viz(&mut program).unwrap()
}
