use std::{fs::File, io::Write};

use clap::{CommandFactory, Parser, Subcommand};
use emulator::Emulator;
use tui::ui::EmulatorVisualizer;

/*
CLI usage:
- TODO: `myprogram <file>.fmem` - visualize
- TODO: `myprogram assemble <file>.asm [-o <output>.fmem]` - assemble
*/

#[derive(Parser, Debug)]
#[command(name = "Flisp Emulator", version, about = "Unified tool for the Flisp Emulator", long_about = None)]
struct Cli {
    #[arg(global = true)]
    fmem_file: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Assemble an assembly file into a .fmem file")]
    Assemble {
        #[arg(short, long, default_value = "output.fmem")]
        output: String,
        input: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    dbg!(&args);

    match args.command {
        Some(Commands::Assemble { input, output }) => {
            // run_assemble(input, output)?;
            todo!();
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

fn run_visualize(fmem: String) {
    let mut program = Emulator::default();
    let mut file = File::open(fmem).expect("Failed to open fmem file");
    todo!();
    // program
    //     .load_memory_from_file(&mut file)
    //     .expect("Failed to load fmem file into memory");

    // EmulatorVisualizer::viz(&mut program).unwrap();
}
