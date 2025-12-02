#![allow(unused, dead_code, unused_variables)]

mod program;
mod program_viz;
mod register;

use std::{fs::File, io::Write};

use program::Program;
use program_viz::ProgramVisualizer;

fn main() {
    let mut program = Program::default();
    let mut data = [0_u8; 256];
    // data[0xFF] = 0x20;
    // data[0x20] = 0xf0;
    // data[0x21] = 0x69;
    // data[0x22] = 0xf1;
    // data[0x23] = 0x69;
    // data[0x69] = 0x67;
    data[0x1f] = 0x07;
    data[0x20] = 0xf0;
    data[0x21] = 0x08;
    data[0x22] = 0xf1;
    data[0x23] = 0x30;
    data[0x30] = 0x13;
    data[0xff] = 0x20;
    program.load_memory(&data);

    let mut file = File::create("output.fmem").unwrap();
    file.write_all(&data).unwrap();

    ProgramVisualizer::viz(&mut program).unwrap();
    // program.execute();
}
