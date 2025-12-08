#![allow(unused, dead_code, unused_variables)]

mod emulator;
mod event;
mod register;
mod state;
mod ui;

use std::{fs::File, io::Write};

use emulator::Emulator;
use ui::EmulatorVisualizer;

fn main() {
    let mut program = Emulator::default();
    let mut data = [0_u8; 256];

    // Start vector
    data[0xff] = 0x20;

    // Array
    data[0x00] = 0x01;
    data[0x01] = 0x02;
    data[0x02] = 0x03;
    data[0x03] = 0x04;

    // Assembly
    data[0x20] = 0x05; // CLRA
    data[0x21] = 0x90; // LDX #Data
    data[0x22] = 0x00; // Data = 0
    data[0x23] = 0x92; // LDSP #Data
    data[0x24] = 0x20; // Data = 0x20

    data[0x25] = 0xc6; // ADDA n,X
    data[0x26] = 0x00; // n = 0
    data[0x27] = 0xc6; // ADDA n,X
    data[0x28] = 0x01; // n = 1
    data[0x29] = 0xc6; // ADDA n,X
    data[0x2a] = 0x02; // n = 2
    data[0x2b] = 0xc6; // ADDA n,X
    data[0x2c] = 0x03; // n = 3
    data[0x2d] = 0x10; // PSHA

    program.load_memory(&data);

    let mut file = File::create("output.fmem").unwrap();
    file.write_all(&data).unwrap();

    EmulatorVisualizer::viz(&mut program).unwrap();
    // program.execute();
}
