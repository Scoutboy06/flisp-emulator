#![allow(unused, dead_code, unused_variables)]

mod program;
mod program_viz;

use std::{fs::File, io::Write};

use program::Program;
use program_viz::ProgramVisualizer;

fn main() {
    let mut program = Program::default();
    let mut data = [0_u8; 256];
    data[0xFF] = 0x20;
    data[0x20] = 0xf0;
    data[0x21] = 0x69;
    program.load_memory(&data);

    // let mut file = File::create("output.fmem").unwrap();
    // file.write_all(&data).unwrap();

    ProgramVisualizer::viz(&mut program).unwrap();
    // program.execute();
}
