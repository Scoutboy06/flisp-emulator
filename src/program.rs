use std::collections::VecDeque;

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Register {
    data: u8,
}

impl Register {
    pub fn new(data: u8) -> Self {
        Self { data }
    }

    pub fn set(&mut self, data: u8) {
        self.data = data;
    }

    pub fn get(&self) -> u8 {
        self.data
    }

    pub fn get_bit(&self, bit_idx: u8) -> bool {
        self.data & (1u8 << bit_idx) != 0
    }

    pub fn inc(&mut self) -> bool {
        let res = self.data.overflowing_add(1);
        self.data = res.0;
        res.1
    }
}

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

    pub fn memory_at(&self, addr: u8) -> u8 {
        self.memory[addr as usize].get()
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

    fn debug_log(&mut self, msg: String) {
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
                self.next_instruction();
                self.reg.pc.inc();
                self.q_state = QState::Fetch;
            }
            QState::Execute => unreachable!(),
        }

        if self.reg.pc.get() >= 100 {
            self.exit();
        }
    }

    fn next_instruction(&mut self) {
        let instruction = self.memory_at(self.reg.pc.get());
        self.debug_log(format!(
            "INS: {:02x}, PC: {:02x}",
            instruction,
            self.reg.pc.get()
        ));
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
            0xf0 => {
                // LDA #Data
                let data = self.memory_at(self.reg.pc.get() + 1);
                self.reg.a.set(data);
                self.reg.pc.inc();

                self.reg.cc.set(CCFlag::N, self.reg.a.get_bit(7));
                self.reg.cc.set(CCFlag::Z, data == 0u8);
                self.reg.cc.disable(CCFlag::V);
            }
            0xf1 => {
                // LDA Addr
                let addr = self.memory_at(self.reg.pc.get() + 1);
                let data = self.memory_at(addr);
                self.reg.a.set(data);
                self.reg.pc.inc();

                self.reg.cc.set(CCFlag::N, self.reg.a.get_bit(7));
                self.reg.cc.set(CCFlag::Z, data == 0u8);
                self.reg.cc.disable(CCFlag::V);
            }
            _ => todo!(" {:02x}", instruction),
        }
    }
}
