use std::collections::VecDeque;

use crate::register::{GetBit, Register};

#[repr(u8)]
pub enum CCFlag {
    I = 0b00010000,
    N = 0b00001000,
    V = 0b00000100,
    Z = 0b00000010,
    C = 0b00000001,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct CCFlags {
    data: u8,
}

impl CCFlags {
    pub fn new(data: u8) -> Self {
        Self { data }
    }

    pub fn get(&self, flag: CCFlag) -> bool {
        (self.data & (flag as u8)) != 0
    }

    pub fn set(&mut self, flag: CCFlag, value: bool) {
        if value {
            self.data |= (flag as u8)
        } else {
            self.data &= !(flag as u8)
        }
    }

    pub fn enable(&mut self, flag: CCFlag) {
        self.set(flag, true);
    }

    pub fn disable(&mut self, flag: CCFlag) {
        self.set(flag, false);
    }
}

#[derive(Default, Copy, Clone)]
pub struct RegisterStore {
    a: Register,
    x: Register,
    y: Register,
    r: Register,
    i: Register,
    sp: Register,
    pc: Register,
    ta: Register,
    cc: CCFlags,
    ld: Register,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum QState {
    Reset,
    Fetch,
    Execute,
}

pub struct Program {
    source_memory: [Register; 256],
    memory: [Register; 256],
    debug_logs: VecDeque<String>,
    reg: RegisterStore,
    q_state: QState,
    clock_count: u32,
    exit: bool,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            source_memory: [Register::default(); 256],
            memory: [Register::default(); 256],
            debug_logs: VecDeque::new(),
            reg: RegisterStore::default(),
            q_state: QState::Reset,
            clock_count: 0,
            exit: false,
        }
    }
}

impl Program {
    pub fn load_memory(&mut self, data: &[u8; 256]) {
        for i in 0..256 {
            self.memory[i] = Register::new(data[i]);
            self.source_memory[i] = Register::new(data[i]);
        }
    }

    pub fn memory(&self) -> &[Register; 256] {
        &self.memory
    }

    pub fn memory_at<T: Into<u8>>(&self, adr: T) -> u8 {
        self.memory[adr.into() as usize].get()
    }

    pub fn reg_a(&self) -> Register {
        self.reg.a
    }
    pub fn reg_x(&self) -> Register {
        self.reg.x
    }
    pub fn reg_y(&self) -> Register {
        self.reg.y
    }
    pub fn reg_r(&self) -> Register {
        self.reg.r
    }
    pub fn reg_sp(&self) -> Register {
        self.reg.sp
    }
    pub fn reg_pc(&self) -> Register {
        self.reg.pc
    }
    pub fn reg_ta(&self) -> Register {
        self.reg.ta
    }
    pub fn reg_cc(&self) -> CCFlags {
        self.reg.cc
    }
    pub fn reg_ld(&self) -> Register {
        self.reg.ld
    }

    pub fn execute(&mut self) {
        while !self.exit {
            self.step();
        }
    }

    pub fn debug_log(&mut self, msg: String) {
        if self.debug_logs.len() >= 20 {
            self.debug_logs.pop_front();
        }
        self.debug_logs.push_back(msg);
    }

    pub fn get_debug_logs(&self) -> &VecDeque<String> {
        &self.debug_logs
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn reset(&mut self) {
        self.q_state = QState::Reset;
        self.memory = self.source_memory.clone();
        self.step();
    }

    pub fn step(&mut self) {
        match self.q_state {
            QState::Reset => {
                let data = self.memory_at(0xff);
                self.debug_log(format!("RESET ({:02x})", data));
                self.reg.pc.set(data);
                self.q_state = QState::Fetch;
            }
            QState::Fetch => {
                self.q_state = QState::Execute;
                self.reg.i.set(self.memory_at(self.reg.pc));
                self.reg.pc.inc();
                self.next_instruction();
                self.q_state = QState::Fetch;
            }
            QState::Execute => unreachable!(),
        }
    }

    fn next_instruction(&mut self) {
        let instruction = self.reg.i.get();
        // self.debug_log(format!(
        //     "INS: {:02x}, PC: {:02x}",
        //     instruction,
        //     self.reg.pc.get(),
        // ));

        let (mem_use, clock_cycles) = get_instruction_size_and_time(instruction);

        if mem_use == 0 || clock_cycles == 0 {
            panic!(
                "Undefined behavior: tried executing invalid instruction: {:02x}",
                instruction
            );
        }

        match instruction {
            0x03 | 0x04 | 0xe0 | 0xdf | 0xef | 0xff => {
                self.debug_log(format!("Invalid instruction: {:02x}", instruction));
            }
            0x0 => {} // NOP
            0x05 => {
                // CLRA
                self.reg.a.set(0);
                self.reg.cc.disable(CCFlag::N);
                self.reg.cc.enable(CCFlag::Z);
                self.reg.cc.disable(CCFlag::V);
                self.reg.cc.disable(CCFlag::C);
            }
            0x90 => {
                // LDX #Data
                let data = self.memory_at(self.reg.pc);
                self.reg.x.set(data);
                self.set_ldx_flags();
            }
            0x95 => {
                // ADCA #Data
                let data = self.memory_at(self.reg.pc);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xa5 => {
                // ADCA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xb5 => {
                // ADCA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xc5 => {
                // ADCA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xd5 => {
                // ADCA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0x96 => {
                // ADDA #Data
                let data = self.memory_at(self.reg.pc);
                let (sum, c, v) = self.reg.a + data;
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, sum == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xa6 => {
                // ADDA Adr
                let adr = self.memory_at(self.reg.pc);
                let (sum, c, v) = self.memory_at(adr) + self.reg.a;
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, sum == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xb6 => {
                // ADDA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (sum, c, v) = self.reg.a + self.memory_at(adr);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, sum == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xc6 => {
                // ADDA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (sum, c, v) = self.reg.a + self.memory_at(adr);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, sum == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xd6 => {
                // ADDA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (sum, c, v) = self.reg.a + self.memory_at(adr);
                self.reg.a.set(sum);
                self.reg.cc.set(CCFlag::N, sum.bit(7));
                self.reg.cc.set(CCFlag::Z, sum == 0);
                self.reg.cc.set(CCFlag::V, v);
                self.reg.cc.set(CCFlag::C, c);
            }
            0xf0 => {
                // LDA #Data
                let data = self.memory_at(self.reg.pc);
                self.reg.a.set(data);
                self.set_lda_flags();
            }
            0xf1 => {
                // LDA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                self.reg.a.set(data);
                self.set_lda_flags();
            }
            0xf2 => {
                // LDA n, SP
                let n = self.memory_at(self.reg.pc);
                let (sum, _, _) = n + self.reg.sp;
                let data = self.memory_at(sum);
                self.reg.a.set(data);
                self.set_lda_flags();
            }
            0xf3 => {
                // LDA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                self.reg.a.set(data);
                self.set_lda_flags();
            }
            0xf4 => {
                // LDA A,X
                let (sum, _) = self.reg.a + self.reg.x;
                self.reg.a.set(self.memory_at(sum));
                self.set_lda_flags();
            }
            0xf5 => {
                // LDA ,X+
                self.reg.a.set(self.memory_at(self.reg.x));
                self.reg.x.inc();
                self.set_lda_flags();
            }
            0xf6 => {
                // LDA ,X-
                self.reg.a.set(self.memory_at(self.reg.x));
                self.reg.x.dec();
                self.set_lda_flags();
            }
            0xf7 => {
                // LDA ,+X
                self.reg.x.inc();
                self.reg.a.set(self.memory_at(self.reg.x));
                self.set_lda_flags();
            }
            0xf8 => {
                // LDA ,-X
                self.reg.x.dec();
                self.reg.a.set(self.memory_at(self.reg.x));
                self.set_lda_flags();
            }
            0xf9 => {
                // LDA n,Y
                let n = self.memory_at(self.reg.pc);
                let (sum, _, _) = n + self.reg.y;
                self.reg.a.set(sum);
                self.set_lda_flags();
            }
            0xfa => {
                // LDA A,Y
                let (sum, _) = self.reg.a + self.reg.y;
                self.reg.a.set(sum);
                self.set_lda_flags();
            }
            0xfb => {
                // LDA ,Y+
                self.reg.a.set(self.memory_at(self.reg.y));
                self.reg.y.inc();
                self.set_lda_flags();
            }
            0xfc => {
                // LDA ,Y-
                self.reg.a.set(self.memory_at(self.reg.y));
                self.reg.y.dec();
                self.set_lda_flags();
            }
            0xfd => {
                // LDA ,+Y
                self.reg.y.inc();
                self.reg.a.set(self.memory_at(self.reg.y));
                self.set_lda_flags();
            }
            0xfe => {
                // LDA ,-Y
                self.reg.y.dec();
                self.reg.a.set(self.memory_at(self.reg.y));
                self.set_lda_flags();
            }
            _ => {
                self.debug_log(format!("Not yet implemented: {:02x}", instruction));
            }
        };

        self.clock_count += clock_cycles as u32;
        let new_pc = (self.reg.pc + (mem_use - 1)).0;
        self.reg.pc.set(new_pc);
    }

    fn set_lda_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.a.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
        self.reg.cc.disable(CCFlag::V);
    }

    fn set_ldx_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.x.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.x == 0);
        self.reg.cc.disable(CCFlag::V);
    }

    fn todo(&mut self, instruction: u8, clk: u32) -> u32 {
        self.debug_log(format!("Not yet implemented: {:02x}", instruction));
        clk
    }
}

/// Returns: (size, clock_cycles)
fn get_instruction_size_and_time(instruction: u8) -> (u8, u8) {
    match instruction {
        0xe0 | 0x03 | 0x04 | 0xdf | 0xef | 0xff => (0, 0),
        0x00 => (1, 2),
        0x01 => (2, 4),
        0x02 => (2, 4),
        0x03 => (0, 0),
        0x04 => (0, 0),
        0x05 => (1, 3),
        0x06 => (1, 3),
        0x07 => (1, 3),
        0x08 => (1, 3),
        0x09 => (1, 2),
        0x0a => (1, 3),
        0x0b => (1, 3),
        0x0c => (1, 3),
        0x0d => (1, 3),
        0x0e => (1, 3),
        0x0f => (1, 3),
        0x10 => (1, 3),
        0x11 => (1, 3),
        0x12 => (1, 3),
        0x13 => (1, 3),
        0x14 => (1, 3),
        0x15 => (1, 3),
        0x16 => (1, 3),
        0x17 => (1, 3),
        0x18 => (1, 2),
        0x19 => (1, 2),
        0x1a => (1, 2),
        0x1b => (1, 2),
        0x1c => (1, 2),
        0x1d => (1, 2),
        0x1e => (1, 2),
        0x1f => (1, 2),
        0x20 => (2, 5),
        0x21 => (2, 4),
        0x22 => (2, 4),
        0x23 => (2, 4),
        0x24 => (2, 4),
        0x25 => (2, 4),
        0x26 => (2, 4),
        0x27 => (2, 4),
        0x28 => (2, 4),
        0x29 => (2, 4),
        0x2a => (2, 4),
        0x2b => (2, 4),
        0x2c => (2, 4),
        0x2d => (2, 4),
        0x2e => (2, 4),
        0x2f => (2, 4),
        0x30 => (2, 3),
        0x31 => (2, 3),
        0x32 => (2, 3),
        0x33 => (2, 2),
        0x34 => (2, 4),
        0x35 => (2, 3),
        0x36 => (2, 4),
        0x37 => (2, 4),
        0x38 => (2, 4),
        0x39 => (2, 3),
        0x3a => (2, 4),
        0x3b => (2, 4),
        0x3c => (2, 4),
        0x3d => (2, 4),
        0x3e => (2, 4),
        0x3f => (2, 4),
        0x40 => (2, 3),
        0x41 => (2, 3),
        0x42 => (2, 3),
        0x43 => (1, 2),
        0x44 => (1, 6),
        0x45 => (2, 3),
        0x46 => (2, 4),
        0x47 => (2, 4),
        0x48 => (2, 4),
        0x49 => (2, 3),
        0x4a => (2, 4),
        0x4b => (2, 4),
        0x4c => (2, 4),
        0x4d => (2, 4),
        0x4e => (2, 4),
        0x4f => (2, 4),
        0x50 => (2, 3),
        0x51 => (2, 3),
        0x52 => (2, 3),
        0x53 => (2, 4),
        0x54 => (2, 5),
        0x55 => (2, 3),
        0x56 => (2, 4),
        0x57 => (2, 4),
        0x58 => (2, 4),
        0x59 => (2, 3),
        0x5a => (2, 4),
        0x5b => (2, 4),
        0x5c => (2, 4),
        0x5d => (2, 4),
        0x5e => (2, 4),
        0x5f => (2, 4),
        0x60 => (1, 3),
        0x61 => (1, 3),
        0x62 => (1, 3),
        0x63 => (1, 4),
        0x64 => (1, 5),
        0x65 => (1, 3),
        0x66 => (1, 4),
        0x67 => (1, 4),
        0x68 => (1, 4),
        0x69 => (1, 3),
        0x6a => (1, 4),
        0x6b => (1, 4),
        0x6c => (1, 4),
        0x6d => (1, 4),
        0x6e => (1, 4),
        0x6f => (1, 4),
        0x70 => (2, 3),
        0x71 => (2, 3),
        0x72 => (2, 3),
        0x73 => (2, 4),
        0x74 => (2, 5),
        0x75 => (2, 3),
        0x76 => (2, 4),
        0x77 => (2, 4),
        0x78 => (2, 4),
        0x79 => (2, 3),
        0x7a => (2, 4),
        0x7b => (2, 4),
        0x7c => (2, 4),
        0x7d => (2, 4),
        0x7e => (2, 4),
        0x7f => (2, 4),
        0x80 => (1, 3),
        0x81 => (1, 3),
        0x82 => (1, 3),
        0x83 => (1, 4),
        0x84 => (1, 5),
        0x85 => (1, 3),
        0x86 => (1, 4),
        0x87 => (1, 4),
        0x88 => (1, 4),
        0x89 => (1, 3),
        0x8a => (1, 4),
        0x8b => (1, 4),
        0x8c => (1, 4),
        0x8d => (1, 4),
        0x8e => (1, 4),
        0x8f => (1, 4),
        0x90 => (2, 2),
        0x91 => (2, 2),
        0x92 => (2, 2),
        0x93 => (2, 4),
        0x94 => (2, 4),
        0x95 => (2, 4),
        0x96 => (2, 4),
        0x97 => (2, 3),
        0x98 => (2, 3),
        0x99 => (2, 4),
        0x9a => (2, 4),
        0x9b => (2, 4),
        0x9c => (2, 3),
        0x9d => (2, 3),
        0x9e => (2, 3),
        0x9f => (1, 4),
        0xa0 => (2, 3),
        0xa1 => (2, 3),
        0xa2 => (2, 3),
        0xa3 => (2, 5),
        0xa4 => (2, 5),
        0xa5 => (2, 5),
        0xa6 => (2, 5),
        0xa7 => (2, 4),
        0xa8 => (2, 4),
        0xa9 => (2, 5),
        0xaa => (2, 5),
        0xab => (2, 5),
        0xac => (2, 4),
        0xad => (2, 4),
        0xae => (2, 4),
        0xaf => (1, 4),
        0xb0 => (2, 3),
        0xb1 => (2, 3),
        0xb2 => (2, 3),
        0xb3 => (2, 5),
        0xb4 => (2, 5),
        0xb5 => (2, 5),
        0xb6 => (2, 5),
        0xb7 => (2, 4),
        0xb8 => (2, 4),
        0xb9 => (2, 5),
        0xba => (2, 5),
        0xbb => (2, 5),
        0xbc => (2, 4),
        0xbd => (2, 4),
        0xbe => (2, 4),
        0xbf => (1, 4),
        0xc0 => (2, 3),
        0xc1 => (2, 3),
        0xc2 => (2, 3),
        0xc3 => (2, 5),
        0xc4 => (2, 5),
        0xc5 => (2, todo!("Instruction 0xC5")),
        0xc6 => (2, 5),
        0xc7 => (2, 4),
        0xc8 => (2, 4),
        0xc9 => (2, 5),
        0xca => (2, 5),
        0xcb => (2, 5),
        0xcc => (2, 4),
        0xcd => (2, 4),
        0xce => (2, 4),
        0xcf => (1, 4),
        0xd0 => (2, 3),
        0xd1 => (2, 3),
        0xd2 => (2, 3),
        0xd3 => (2, 5),
        0xd4 => (2, 5),
        0xd5 => (2, 5),
        0xd6 => (2, 5),
        0xd7 => (2, 4),
        0xd8 => (2, 4),
        0xd9 => (2, 5),
        0xda => (2, 5),
        0xdb => (2, 5),
        0xdc => (2, 4),
        0xdd => (2, 4),
        0xde => (2, 4),
        0xdf => (0, 0),
        0xe0 => (0, 0),
        0xe1 => (2, 3),
        0xe2 => (2, 3),
        0xe3 => (2, 3),
        0xe4 => (1, 3),
        0xe5 => (1, 4),
        0xe6 => (1, 4),
        0xe7 => (1, 4),
        0xe8 => (1, 4),
        0xe9 => (2, 3),
        0xea => (1, 3),
        0xeb => (1, 4),
        0xec => (1, 4),
        0xed => (1, 4),
        0xee => (1, 4),
        0xef => (0, 0),
        0xf0 => (2, 2),
        0xf1 => (2, 3),
        0xf2 => (2, 3),
        0xf3 => (2, 3),
        0xf4 => (1, 3),
        0xf5 => (1, 4),
        0xf6 => (1, 4),
        0xf7 => (1, 4),
        0xf8 => (1, 4),
        0xf9 => (2, 3),
        0xfa => (1, 3),
        0xfb => (1, 4),
        0xfc => (1, 4),
        0xfd => (1, 4),
        0xfe => (1, 4),
        0xff => (0, 0),
    }
}
