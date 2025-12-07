use std::collections::VecDeque;

use crate::register::{
    GetBit, Register, add, rotate_left, rotate_right, shl, shr, shr_signed, sub, sub_c,
};

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

    pub fn overwrite(&mut self, data: u8) {
        self.data = data;
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
    clk_count: u32,
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
            clk_count: 0,
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

    pub fn clk_count(&self) -> u32 {
        self.clk_count
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
        self.clk_count = 0;
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
            panic!("Tried executing invalid instruction: {:02x}", instruction);
        }

        match instruction {
            0x03 | 0x04 | 0xe0 | 0xdf | 0xef | 0xff => {
                self.debug_log(format!("Invalid instruction: {:02x}", instruction));
            }
            0x00 => {} // NOP
            0x01 => {
                // ANDCC #Data
                let data = self.memory_at(self.reg.pc);
                let result = self.reg.cc.data & data;
                self.reg.cc.overwrite(result);
            }
            0x02 => {
                // ORCC #Data
                let data = self.memory_at(self.reg.pc);
                let result = self.reg.cc.data | data;
                self.reg.cc.overwrite(result);
            }
            0x05 => {
                // CLRA
                self.reg.a.set(0);
                self.set_clr_flags();
            }
            0x06 => {
                // NEGA
                let (new_a, _c, v) = sub(0, self.reg.a.get());
                self.set_neg_flags(new_a, self.reg.a.get(), v);
                self.reg.a.set(new_a);
            }
            0x07 => {
                // INCA
                let (_c, v) = self.reg.a.inc();
                let new_a = self.reg.a.get();
                self.set_inc_flags(new_a, v);
            }
            0x08 => {
                // DECA
                let (_c, v) = self.reg.a.dec();
                self.set_dec_flags(self.reg.a.get(), v);
            }
            0x09 => {
                // TSTA
                self.set_tst_flags(self.reg.a.get());
            }
            0x10 => {
                // PSHA
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.a);
            }
            0x11 => {
                // PSHX
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.x);
            }
            0x12 => {
                // PSHY
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.y);
            }
            0x13 => {
                // PSHC
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.cc.data);
            }
            0x14 => {
                // PULA
                let val = self.memory_at(self.reg.sp);
                self.reg.a.set(val);
                self.reg.sp.inc();
            }
            0x15 => {
                // PULX
                let val = self.memory_at(self.reg.sp);
                self.reg.x.set(val);
                self.reg.sp.inc();
            }
            0x16 => {
                // PULY
                let val = self.memory_at(self.reg.sp);
                self.reg.y.set(val);
                self.reg.sp.inc();
            }
            0x17 => {
                // PULC
                let val = self.memory_at(self.reg.sp);
                self.reg.cc.overwrite(val);
                self.reg.sp.inc();
            }
            0x18 => {
                // TFR A,CC
                self.reg.cc.overwrite(self.reg.a.get());
            }
            0x19 => {
                // TFR CC,A
                self.reg.a.set(self.reg.cc.data);
            }
            0x1a => {
                // TFR X,Y
                self.reg.y.set(self.reg.x.get());
            }
            0x1b => {
                // TFR Y,X
                self.reg.x.set(self.reg.y.get());
            }
            0x1c => {
                // TFR X,SP
                self.reg.sp.set(self.reg.x.get());
            }
            0x1d => {
                // TFR SP,X
                self.reg.x.set(self.reg.sp.get());
            }
            0x1e => {
                // TFR Y,SP
                self.reg.sp.set(self.reg.y.get());
            }
            0x1f => {
                // TFR SP,Y
                self.reg.y.set(self.reg.sp.get());
            }
            0x0a => {
                // COMA
                let new_a = !self.reg.a.get();
                self.reg.a.set(new_a);
                self.set_com_flags(new_a);
            }
            0x0b => {
                // ASLA / LSLA
                let (new_a, c, v) = shl(self.reg.a);
                self.reg.a.set(new_a);
                self.set_asl_flags(new_a, c, v);
            }
            0x0c => {
                // LSRA
                let (new_a, c, v) = shr(self.reg.a);
                self.reg.a.set(new_a);
                self.set_lsr_flags(new_a, c, v);
            }
            0x0d => {
                // ROLA
                let (new_a, c) = rotate_left(self.reg.a);
                self.reg.a.set(new_a);
                self.set_rol_flags(new_a, c);
            }
            0x0e => {
                // RORA
                let (new_a, c) = rotate_right(self.reg.a);
                self.reg.a.set(new_a);
                self.set_ror_flags(new_a, c);
            }
            0x0f => {
                // ASRA
                let (new_a, c) = shr_signed(self.reg.a.get());
                self.reg.a.set(new_a);
                self.set_asr_flags(new_a, c);
            }
            0x20 => {
                // BSR Adr
                self.reg.sp.dec();
                let return_addr = self.reg.pc.get();
                self.memory[self.reg.sp.get() as usize].set(return_addr);
                let offset = self.memory_at(self.reg.pc);
                let (new_pc, _, _) = self.reg.pc + offset;
                self.reg.pc.set(new_pc);
            }
            0x21 => {
                // BRA Adr
                let offset = self.memory_at(self.reg.pc);
                let (new_pc, _, _) = self.reg.pc + offset;
                self.reg.pc.set(new_pc);
            }
            0x22 => {
                // BMI Adr
                if self.reg.cc.get(CCFlag::N) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x23 => {
                // BPL Adr
                if !self.reg.cc.get(CCFlag::N) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x24 => {
                // BEQ Adr
                let z = self.reg.cc.get(CCFlag::Z);
                if z {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x25 => {
                // BNE Adr
                if !self.reg.cc.get(CCFlag::Z) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x26 => {
                // BVS Adr
                if self.reg.cc.get(CCFlag::V) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x27 => {
                // BVC Adr
                if !self.reg.cc.get(CCFlag::V) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x28 => {
                // BCS Adr
                let c = self.reg.cc.get(CCFlag::C);
                if c {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x29 => {
                // BCC Adr
                let c = self.reg.cc.get(CCFlag::C);
                if !c {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x2a => {
                // BHI Adr
                let c = self.reg.cc.get(CCFlag::C);
                let z = self.reg.cc.get(CCFlag::Z);
                if !(c || z) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x2b => {
                // BLS Adr
                let c = self.reg.cc.get(CCFlag::C);
                let z = self.reg.cc.get(CCFlag::Z);
                if c || z {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x2c => {
                // BGT Adr
                let n = self.reg.cc.get(CCFlag::N);
                let v = self.reg.cc.get(CCFlag::V);
                let z = self.reg.cc.get(CCFlag::Z);
                if !(n != v || z) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x2d => {
                // BGE Adr
                if self.reg.cc.get(CCFlag::N) == self.reg.cc.get(CCFlag::V) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x2e => {
                // BLE Adr
                let n = self.reg.cc.get(CCFlag::N);
                let v = self.reg.cc.get(CCFlag::V);
                let z = self.reg.cc.get(CCFlag::Z);
                if n != v || z {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x2f => {
                // BLT Adr
                if self.reg.cc.get(CCFlag::N) != self.reg.cc.get(CCFlag::V) {
                    let offset = self.memory_at(self.reg.pc);
                    let (new_pc, _, _) = self.reg.pc + offset;
                    self.reg.pc.set(new_pc);
                }
            }
            0x30 => {
                // STX Adr
                let adr = self.memory_at(self.reg.pc);
                self.memory[adr as usize].set(self.reg.x.get());
            }
            0x31 => {
                // STY Adr
                let adr = self.memory_at(self.reg.pc);
                self.memory[adr as usize].set(self.reg.y.get());
            }
            0x32 => {
                // STSP Adr
                let adr = self.memory_at(self.reg.pc);
                self.memory[adr as usize].set(self.reg.sp.get());
            }
            0x33 => {
                // JMP Adr
                let adr = self.memory_at(self.reg.pc);
                self.reg.pc.set(adr);
            }
            0x34 => {
                // JSR Adr
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.pc);
                let adr = self.memory_at(self.reg.pc);
                self.reg.pc.set(adr);
            }
            0x35 => {
                // CLR Adr
                let adr = self.memory_at(self.reg.pc);
                self.memory[adr as usize].set(0);
                self.set_clr_flags();
            }
            0x36 => {
                // NEG Adr
                let adr = self.memory_at(self.reg.pc);
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(0, val);
                self.memory[adr as usize].set(new_val);
                self.set_neg_flags(new_val, val, v);
            }
            0x37 => {
                // INC Adr
                let adr = self.memory_at(self.reg.pc);
                let val = self.memory_at(adr);
                let (new_val, _c, v) = add(val, 1, false);
                self.memory[adr as usize].set(new_val);
                self.set_inc_flags(new_val, v);
            }
            0x38 => {
                // DEC Adr
                let adr = self.memory_at(self.reg.pc);
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(val, 1);
                self.memory[adr as usize].set(new_val);
                self.set_dec_flags(new_val, v);
            }
            0x39 => {
                // TST Adr
                let adr = self.memory_at(self.reg.pc);
                let val = self.memory_at(adr);
                self.set_tst_flags(val);
            }
            0x3a => {
                // COM Adr
                let adr = self.memory_at(self.reg.pc);
                let new_val = !self.memory_at(adr);
                self.memory[adr as usize].set(new_val);
                self.set_com_flags(new_val);
            }
            0x3b => {
                // ASL Adr / LSL Adr
                let adr = self.memory_at(self.reg.pc);
                let (new_val, c, v) = shl(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asl_flags(new_val, c, v);
            }
            0x3c => {
                // LSR Adr
                let adr = self.memory_at(self.reg.pc);
                let (new_val, c, v) = shr(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_lsr_flags(new_val, c, v);
            }
            0x3d => {
                // ROL Adr
                let adr = self.memory_at(self.reg.pc);
                let (new_val, c) = rotate_left(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_rol_flags(new_val, c);
            }
            0x3e => {
                // ROR Adr
                let adr = self.memory_at(self.reg.pc);
                let (new_val, c) = rotate_right(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_ror_flags(new_val, c);
            }
            0x3f => {
                // ASR Adr
                let adr = self.memory_at(self.reg.pc);
                let (new_val, c) = shr_signed(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asr_flags(new_val, c);
            }
            0x40 => {
                // STX n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.memory[adr as usize].set(self.reg.x.get());
            }
            0x41 => {
                // STY n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.memory[adr as usize].set(self.reg.y.get());
            }
            0x42 => {
                // STSP n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.memory[adr as usize].set(self.reg.sp.get());
            }
            0x43 => {
                // RTS
                let return_addr = self.memory_at(self.reg.sp);
                self.reg.pc.set(return_addr);
                self.reg.sp.inc();
            }
            0x44 => {
                // RTI
                self.reg.cc.overwrite(self.memory_at(self.reg.sp));
                self.reg.sp.inc();
                self.reg.a.set(self.memory_at(self.reg.sp));
                self.reg.sp.inc();
                self.reg.x.set(self.memory_at(self.reg.sp));
                self.reg.sp.inc();
                self.reg.y.set(self.memory_at(self.reg.sp));
                self.reg.sp.inc();
                self.reg.pc.set(self.memory_at(self.reg.sp));
                self.reg.sp.inc();
            }
            0x45 => {
                // CLR n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.memory[adr as usize].set(0);
                self.set_clr_flags();
            }
            0x46 => {
                // NEG n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(0, val);
                self.memory[adr as usize].set(new_val);
                self.set_neg_flags(new_val, val, v);
            }
            0x47 => {
                // INC n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = add(val, 1, false);
                self.memory[adr as usize].set(new_val);
                self.set_inc_flags(new_val, v);
            }
            0x48 => {
                // DEC n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(val, 1);
                self.memory[adr as usize].set(new_val);
                self.set_dec_flags(new_val, v);
            }
            0x49 => {
                // TST n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let val = self.memory_at(adr);
                self.set_tst_flags(val);
            }
            0x4a => {
                // COM n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let new_val = !self.memory_at(adr);
                self.memory[adr as usize].set(new_val);
                self.set_com_flags(new_val);
            }
            0x4b => {
                // ASL n,SP / LSL n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (new_val, c, v) = shl(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asl_flags(new_val, c, v);
            }
            0x4c => {
                // LSR n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (new_val, c, v) = shr(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_lsr_flags(new_val, c, v);
            }
            0x4d => {
                // ROL n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (new_val, c) = rotate_left(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_rol_flags(new_val, c);
            }
            0x4e => {
                // ROR n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (new_val, c) = rotate_right(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_ror_flags(new_val, c);
            }
            0x4f => {
                // ASR n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (new_val, c) = shr_signed(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asr_flags(new_val, c);
            }
            0x50 => {
                // STX n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.memory[adr as usize].set(self.reg.x.get());
            }
            0x51 => {
                // STY n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.memory[adr as usize].set(self.reg.y.get());
            }
            0x52 => {
                // STSP n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.memory[adr as usize].set(self.reg.sp.get());
            }
            0x53 => {
                // JMP n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.pc.set(adr);
            }
            0x54 => {
                // JSR n,X
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.pc);
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.pc.set(adr);
            }
            0x55 => {
                // CLR n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.memory[adr as usize].set(0);
                self.set_clr_flags();
            }
            0x56 => {
                // NEG n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(0, val);
                self.memory[adr as usize].set(new_val);
                self.set_neg_flags(new_val, val, v);
            }
            0x57 => {
                // INC n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = add(val, 1, false);
                self.memory[adr as usize].set(new_val);
                self.set_inc_flags(new_val, v);
            }
            0x58 => {
                // DEC n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(val, 1);
                self.memory[adr as usize].set(new_val);
                self.set_dec_flags(new_val, v);
            }
            0x59 => {
                // TST n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let val = self.memory_at(adr);
                self.set_tst_flags(val);
            }
            0x5a => {
                // COM n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let new_val = !self.memory_at(adr);
                self.memory[adr as usize].set(new_val);
                self.set_com_flags(new_val);
            }
            0x5b => {
                // ASL n,X / LSL n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (new_val, c, v) = shl(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asl_flags(new_val, c, v);
            }
            0x5c => {
                // LSR n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (new_val, c, v) = shr(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_lsr_flags(new_val, c, v);
            }
            0x5d => {
                // ROL n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (new_val, c) = rotate_left(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_rol_flags(new_val, c);
            }
            0x5e => {
                // ROR n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (new_val, c) = rotate_right(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_ror_flags(new_val, c);
            }
            0x5f => {
                // ASR n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (new_val, c) = shr_signed(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asr_flags(new_val, c);
            }
            0x60 => {
                // STX A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                self.memory[adr as usize].set(self.reg.x.get());
            }
            0x61 => {
                // TODO: FLISP-hanbook said OP-code 60, but I assume it should be 61.
                // STY A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                self.memory[adr as usize].set(self.reg.y.get());
            }
            0x62 => {
                // STSP A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                self.memory[adr as usize].set(self.reg.sp.get());
            }
            0x63 => {
                // JMP A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                self.reg.pc.set(adr);
            }
            0x64 => {
                // JSR A,X
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.pc);
                let (adr, _, _) = self.reg.a + self.reg.x;
                self.reg.pc.set(adr);
            }
            0x67 => {
                // INC A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = add(val, 1, false);
                self.memory[adr as usize].set(new_val);
                self.set_inc_flags(new_val, v);
            }
            0x65 => {
                // CLR A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                self.memory[adr as usize].set(0);
                self.set_clr_flags();
            }
            0x66 => {
                // NEG A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(0, val);
                self.memory[adr as usize].set(new_val);
                self.set_neg_flags(new_val, val, v);
            }
            0x68 => {
                // DEC A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(val, 1);
                self.memory[adr as usize].set(new_val);
                self.set_dec_flags(new_val, v);
            }
            0x69 => {
                // TST A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let val = self.memory_at(adr);
                self.set_tst_flags(val);
            }
            0x6a => {
                // COM A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let new_val = !self.memory_at(adr);
                self.memory[adr as usize].set(new_val);
                self.set_com_flags(new_val);
            }
            0x6b => {
                // ASL A,X / LSL A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let (new_val, c, v) = shl(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asl_flags(new_val, c, v);
            }
            0x6c => {
                // LSR A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let (new_val, c, v) = shr(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_lsr_flags(new_val, c, v);
            }
            0x6d => {
                // ROL A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let (new_val, c) = rotate_left(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_rol_flags(new_val, c);
            }
            0x6e => {
                // ROR A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let (new_val, c) = rotate_right(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_ror_flags(new_val, c);
            }
            0x6f => {
                // ASR A,X
                let (adr, _, _) = self.reg.a + self.reg.x;
                let (new_val, c) = shr_signed(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asr_flags(new_val, c);
            }
            0x70 => {
                // STX n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.memory[adr as usize].set(self.reg.x.get());
            }
            0x71 => {
                // STY n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.memory[adr as usize].set(self.reg.y.get());
            }
            0x72 => {
                // STSP n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.memory[adr as usize].set(self.reg.sp.get());
            }
            0x73 => {
                // JMP n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.pc.set(adr);
            }
            0x74 => {
                // JSR n,Y
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.pc);
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.pc.set(adr);
            }
            0x75 => {
                // CLR n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.memory[adr as usize].set(0);
                self.set_clr_flags();
            }
            0x76 => {
                // NEG n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(0, val);
                self.memory[adr as usize].set(new_val);
                self.set_neg_flags(new_val, val, v);
            }
            0x77 => {
                // INC n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = add(val, 1, false);
                self.memory[adr as usize].set(new_val);
                self.set_inc_flags(new_val, v);
            }
            0x78 => {
                // DEC n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(val, 1);
                self.memory[adr as usize].set(new_val);
                self.set_dec_flags(new_val, v);
            }
            0x79 => {
                // TST n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let val = self.memory_at(adr);
                self.set_tst_flags(val);
            }
            0x7a => {
                // COM n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let new_val = !self.memory_at(adr);
                self.memory[adr as usize].set(new_val);
                self.set_com_flags(new_val);
            }
            0x7b => {
                // ASL n,Y / LSL n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (new_val, c, v) = shl(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asl_flags(new_val, c, v);
            }
            0x7c => {
                // LSR n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (new_val, c, v) = shr(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_lsr_flags(new_val, c, v);
            }
            0x7d => {
                // ROL n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (new_val, c) = rotate_left(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_rol_flags(new_val, c);
            }
            0x7e => {
                // ROR n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (new_val, c) = rotate_right(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_ror_flags(new_val, c);
            }
            0x7f => {
                // ASR n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (new_val, c) = shr_signed(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asr_flags(new_val, c);
            }
            0x80 => {
                // STX A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                self.memory[adr as usize].set(self.reg.x.get());
            }
            0x81 => {
                // STY A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                self.memory[adr as usize].set(self.reg.y.get());
            }
            0x82 => {
                // STSP A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                self.memory[adr as usize].set(self.reg.sp.get());
            }
            0x83 => {
                // JMP A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                self.reg.pc.set(adr);
            }
            0x84 => {
                // JSR A,Y
                self.reg.sp.dec();
                self.memory[self.reg.sp.get() as usize].set(self.reg.pc);
                let (adr, _, _) = self.reg.a + self.reg.y;
                self.reg.pc.set(adr);
            }
            0x85 => {
                // CLR A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                self.memory[adr as usize].set(0);
                self.set_clr_flags();
            }
            0x86 => {
                // NEG A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(0, val);
                self.memory[adr as usize].set(new_val);
                self.set_neg_flags(new_val, val, v);
            }
            0x87 => {
                // INC A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = add(val, 1, false);
                self.memory[adr as usize].set(new_val);
                self.set_inc_flags(new_val, v);
            }
            0x88 => {
                // DEC A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let val = self.memory_at(adr);
                let (new_val, _c, v) = sub(val, 1);
                self.memory[adr as usize].set(new_val);
                self.set_dec_flags(new_val, v);
            }
            0x89 => {
                // TST A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let val = self.memory_at(adr);
                self.set_tst_flags(val);
            }
            0x8a => {
                // COM A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let new_val = !self.memory_at(adr);
                self.memory[adr as usize].set(new_val);
                self.set_com_flags(new_val);
            }
            0x8b => {
                // ASL A,Y / LSL A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let (new_val, c, v) = shl(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asl_flags(new_val, c, v);
            }
            0x8c => {
                // LSR A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let (new_val, c, v) = shr(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_lsr_flags(new_val, c, v);
            }
            0x8d => {
                // ROL A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let (new_val, c) = rotate_left(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_rol_flags(new_val, c);
            }
            0x8e => {
                // ROR A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let (new_val, c) = rotate_right(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_ror_flags(new_val, c);
            }
            0x8f => {
                // ASR A,Y
                let (adr, _, _) = self.reg.a + self.reg.y;
                let (new_val, c) = shr_signed(self.memory_at(adr));
                self.memory[adr as usize].set(new_val);
                self.set_asr_flags(new_val, c);
            }
            0x90 => {
                // LDX #Data
                let data = self.memory_at(self.reg.pc);
                self.reg.x.set(data);
                self.set_ldx_flags();
            }
            0x91 => {
                // LDY #Data
                let data = self.memory_at(self.reg.pc);
                self.reg.y.set(data);
                self.set_ldy_flags();
            }
            0x92 => {
                // LDSP #Data
                let data = self.memory_at(self.reg.pc);
                self.reg.sp.set(data);
                self.set_ldsp_flags();
            }
            0x93 => {
                // SBCA #Data
                let data = self.memory_at(self.reg.pc);
                let (diff, c, v) = sub_c(self.reg.a, data, self.reg.cc.get(CCFlag::C));
                self.reg.a.set(diff);
                self.set_sbc_flags(diff, c, v);
            }
            0x94 => {
                // SUBA #Data
                let data = self.memory_at(self.reg.pc);
                let (diff, c, v) = sub(self.reg.a, data);
                self.reg.a.set(diff);
                self.set_suba_flags(diff, c, v);
            }
            0x95 => {
                // ADCA #Data
                let data = self.memory_at(self.reg.pc);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0x97 => {
                // CMPA #Data
                let data = self.memory_at(self.reg.pc);
                let (diff, c, v) = sub(self.reg.a, data);
                self.set_cmp_flags(diff, c, v);
            }
            0x98 => {
                // BITA #Data
                let data = self.memory_at(self.reg.pc);
                let result = self.reg.a & data;
                self.set_bita_flags(result);
            }
            0x96 => {
                // ADDA #Data
                let data = self.memory_at(self.reg.pc);
                let (sum, c, v) = self.reg.a + data;
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0x99 => {
                // ANDA #Data
                let data = self.memory_at(self.reg.pc);
                let result = self.reg.a & data;
                self.reg.a.set(result);
                self.set_anda_flags();
            }
            0x9a => {
                // ORA #Data
                let data = self.memory_at(self.reg.pc);
                let result = self.reg.a.get() | data;
                self.reg.a.set(result);
                self.set_ora_flags(result);
            }
            0x9b => {
                // EORA #Data
                let data = self.memory_at(self.reg.pc);
                let result = self.reg.a.get() ^ data;
                self.reg.a.set(result);
                self.set_eora_flags(result);
            }
            0x9c => {
                // CMPX #Data
                let data = self.memory_at(self.reg.pc);
                let (diff, c, v) = sub(self.reg.x, data);
                self.set_cmp_flags(diff, c, v);
            }
            0x9d => {
                // CMPY #Data
                let data = self.memory_at(self.reg.pc);
                let (diff, c, v) = sub(self.reg.y, data);
                self.set_cmp_flags(diff, c, v);
            }
            0x9e => {
                // CMPSP #Data
                let data = self.memory_at(self.reg.pc);
                let (diff, c, v) = sub(self.reg.sp, data);
                self.set_cmp_flags(diff, c, v);
            }
            0x9f => {
                // EXG A,CC
                let temp = self.reg.a.get();
                self.reg.a.set(self.reg.cc.data);
                self.reg.cc.data = temp & 0b1111; // Keep only lower 4 bits (N,Z,V,C)
            }
            0xa0 => {
                // LDX Adr
                let adr = self.memory_at(self.reg.pc);
                self.reg.x.set(self.memory_at(adr));
                self.set_ldx_flags();
            }
            0xa1 => {
                // LDY Adr
                let adr = self.memory_at(self.reg.pc);
                self.reg.y.set(self.memory_at(adr));
                self.set_ldy_flags();
            }
            0xa2 => {
                // LDSP Adr
                let adr = self.memory_at(self.reg.pc);
                self.reg.sp.set(self.memory_at(adr));
                self.set_ldsp_flags();
            }
            0xa3 => {
                // SBCA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (diff, c, v) = sub_c(self.reg.a, data, self.reg.cc.get(CCFlag::C));
                self.reg.a.set(diff);
                self.set_sbc_flags(diff, c, v);
            }
            0xa4 => {
                // SUBA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.reg.a.set(diff);
                self.set_suba_flags(diff, c, v);
            }
            0xa5 => {
                // ADCA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xa6 => {
                // ADDA Adr
                let adr = self.memory_at(self.reg.pc);
                let (sum, c, v) = self.memory_at(adr) + self.reg.a;
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xa7 => {
                // CMPA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xa8 => {
                // BITA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let result = self.reg.a & data;
                self.set_bita_flags(result);
            }
            0xa9 => {
                // ANDA Adr
                let adr = self.memory_at(self.reg.pc);
                let result = self.reg.a & self.memory_at(adr);
                self.reg.a.set(result);
                self.set_anda_flags();
            }
            0xaa => {
                // ORA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let result = self.reg.a.get() | data;
                self.reg.a.set(result);
                self.set_ora_flags(result);
            }
            0xab => {
                // EORA Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let result = self.reg.a.get() ^ data;
                self.reg.a.set(result);
                self.set_eora_flags(result);
            }
            0xac => {
                // CMPX Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.x, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xad => {
                // CMPY Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.y, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xae => {
                // CMPSP Adr
                let adr = self.memory_at(self.reg.pc);
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.sp, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xaf => {
                // EXG X,Y
                let temp = self.reg.x.get();
                self.reg.x.set(self.reg.y.get());
                self.reg.y.set(temp);
            }
            0xb0 => {
                // LDX n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.reg.x.set(self.memory_at(adr));
                self.set_ldx_flags();
            }
            0xb1 => {
                // LDY n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.reg.y.set(self.memory_at(adr));
                self.set_ldy_flags();
            }
            0xb2 => {
                // LDSP n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.reg.sp.set(self.memory_at(adr));
                self.set_ldsp_flags();
            }
            0xb3 => {
                // SBCA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub_c(self.reg.a, data, self.reg.cc.get(CCFlag::C));
                self.reg.a.set(diff);
                self.set_sbc_flags(diff, c, v);
            }
            0xb4 => {
                // SUBA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.reg.a.set(diff);
                self.set_suba_flags(diff, c, v);
            }
            0xb5 => {
                // ADCA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xb6 => {
                // ADDA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let (sum, c, v) = self.reg.a + self.memory_at(adr);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xb7 => {
                // CMPA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xb8 => {
                // BITA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                self.set_bita_flags(self.reg.a & data);
            }
            0xb9 => {
                // ANDA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let result = self.reg.a & data;
                self.reg.a.set(result);
                self.set_anda_flags();
            }
            0xba => {
                // ORA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let result = self.reg.a.get() | data;
                self.reg.a.set(result);
                self.set_ora_flags(result);
            }
            0xbb => {
                // EORA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let result = self.reg.a.get() ^ data;
                self.reg.a.set(result);
                self.set_eora_flags(result);
            }
            0xbc => {
                // CMPX n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.x, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xbd => {
                // CMPY n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.y, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xbe => {
                // LEASP n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.reg.sp.set(adr);
            }
            0xbf => {
                // EXG X,SP
                let temp = self.reg.x.get();
                self.reg.x.set(self.reg.sp.get());
                self.reg.sp.set(temp);
            }
            0xc0 => {
                // LDX n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.x.set(self.memory_at(adr));
                self.set_ldx_flags();
            }
            0xc1 => {
                // LDY n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.y.set(self.memory_at(adr));
                self.set_ldy_flags();
            }
            0xc2 => {
                // LDSP n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.sp.set(self.memory_at(adr));
                self.set_ldsp_flags();
            }
            0xc3 => {
                // SBCA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub_c(self.reg.a, data, self.reg.cc.get(CCFlag::C));
                self.reg.a.set(diff);
                self.set_sbc_flags(diff, c, v);
            }
            0xc4 => {
                // SUBA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.reg.a.set(diff);
                self.set_suba_flags(diff, c, v);
            }
            0xc5 => {
                // ADCA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xc6 => {
                // ADDA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let (sum, c, v) = self.reg.a + self.memory_at(adr);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xc7 => {
                // CMPA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xc8 => {
                // BITA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let result = self.reg.a & data;
                self.set_bita_flags(result);
            }
            0xc9 => {
                // ANDA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let result = self.reg.a & data;
                self.reg.a.set(result);
                self.set_anda_flags();
            }
            0xca => {
                // ORA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let result = self.reg.a.get() | data;
                self.reg.a.set(result);
                self.set_ora_flags(result);
            }
            0xcb => {
                // EORA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                let data = self.memory_at(adr);
                let result = self.reg.a.get() ^ data;
                self.reg.a.set(result);
                self.set_eora_flags(result);
            }
            0xcc => {
                // LEAX n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.x.set(adr);
            }
            0xcd => {
                // LEAY n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.y.set(adr);
            }
            0xce => {
                // LEASP n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.reg.sp.set(adr);
            }
            0xcf => {
                // EXG Y,SP
                let temp = self.reg.y.get();
                self.reg.y.set(self.reg.sp.get());
                self.reg.sp.set(temp);
            }
            0xd0 => {
                // LDX n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.x.set(self.memory_at(adr));
                self.set_ldx_flags();
            }
            0xd1 => {
                // LDY n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.y.set(self.memory_at(adr));
                self.set_ldy_flags();
            }
            0xd2 => {
                // LDSP n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.sp.set(self.memory_at(adr));
                self.set_ldsp_flags();
            }
            0xd3 => {
                // SBCA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub_c(self.reg.a, data, self.reg.cc.get(CCFlag::C));
                self.reg.a.set(diff);
                self.set_sbc_flags(diff, c, v);
            }
            0xd4 => {
                // SUBA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.reg.a.set(diff);
                self.set_suba_flags(diff, c, v);
            }
            0xd5 => {
                // ADCA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let (sum, c, v) = self.reg.a.add_c(data);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xd6 => {
                // ADDA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let (sum, c, v) = self.reg.a + self.memory_at(adr);
                self.reg.a.set(sum);
                self.set_add_flags(sum, c, v);
            }
            0xd7 => {
                // CMPA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let (diff, c, v) = sub(self.reg.a, data);
                self.set_cmp_flags(diff, c, v);
            }
            0xd8 => {
                // BITA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let result = self.reg.a & data;
                self.set_bita_flags(result);
            }
            0xd9 => {
                // ANDA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let result = self.reg.a & data;
                self.reg.a.set(result);
                self.set_anda_flags();
            }
            0xda => {
                // ORA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let result = self.reg.a.get() | data;
                self.reg.a.set(result);
                self.set_ora_flags(result);
            }
            0xdb => {
                // EORA n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                let data = self.memory_at(adr);
                let result = self.reg.a.get() ^ data;
                self.reg.a.set(result);
                self.set_eora_flags(result);
            }
            0xdc => {
                // LEAX n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.reg.x.set(adr);
            }
            0xdd => {
                // LEAY n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.reg.y.set(adr);
            }
            0xde => {
                // LEASP n,Y
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.y;
                self.reg.sp.set(adr);
            }
            0xe1 => {
                // STA Adr
                let adr = self.memory_at(self.reg.pc);
                self.memory[adr as usize].set(self.reg.a);
            }
            0xe2 => {
                // STA n,SP
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.sp;
                self.memory[adr as usize].set(self.reg.a);
            }
            0xe3 => {
                // STA n,X
                let n = self.memory_at(self.reg.pc);
                let (adr, _, _) = n + self.reg.x;
                self.memory[adr as usize].set(self.reg.a);
            }
            0xe4 => {
                // STA A,X
                let (sum, _, _) = self.reg.a + self.reg.x;
                self.memory[sum as usize].set(self.reg.a);
            }
            0xe5 => {
                // STA ,X+
                self.memory[self.reg.x.get() as usize].set(self.reg.a);
                self.reg.x.inc();
            }
            0xe6 => {
                // STA ,X-
                self.memory[self.reg.x.get() as usize].set(self.reg.a);
                self.reg.x.dec();
            }
            0xe7 => {
                // STA ,+X
                self.reg.x.inc();
                self.memory[self.reg.x.get() as usize].set(self.reg.a);
            }
            0xe8 => {
                // STA ,-X
                self.reg.x.dec();
                self.memory[self.reg.x.get() as usize].set(self.reg.a);
            }
            0xe9 => {
                // STA n,Y
                let n = self.memory_at(self.reg.pc);
                let (sum, _, _) = n + self.reg.y;
                self.memory[sum as usize].set(self.reg.a);
            }
            0xea => {
                // STA A,Y
                let (sum, _, _) = self.reg.a + self.reg.y;
                self.memory[sum as usize].set(self.reg.a);
            }
            0xeb => {
                // STA ,Y+
                self.memory[self.reg.y.get() as usize].set(self.reg.a);
                self.reg.y.inc();
            }
            0xec => {
                // STA ,Y-
                self.memory[self.reg.y.get() as usize].set(self.reg.a);
                self.reg.y.dec();
            }
            0xed => {
                // STA ,+Y
                self.reg.y.inc();
                self.memory[self.reg.y.get() as usize].set(self.reg.a);
            }
            0xee => {
                // STA ,-Y
                self.reg.y.dec();
                self.memory[self.reg.y.get() as usize].set(self.reg.a);
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
                let (sum, _, _) = self.reg.a + self.reg.x;
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
                let (sum, _, _) = self.reg.a + self.reg.y;
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
        };

        self.clk_count += clock_cycles as u32;
        let new_pc = (self.reg.pc + (mem_use - 1)).0;
        self.reg.pc.set(new_pc);
    }

    fn set_add_flags(&mut self, result: u8, c: bool, v: bool) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.set(CCFlag::C, c);
        self.reg.cc.set(CCFlag::V, v);
    }

    fn set_suba_flags(&mut self, result: u8, c: bool, v: bool) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.set(CCFlag::V, v);
        self.reg.cc.set(CCFlag::C, c);
    }

    fn set_lda_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.a.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by LDA
    }

    fn set_ldx_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.x.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.x == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by LDX
    }

    fn set_eora_flags(&mut self, result: u8) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by EORA
    }

    fn set_ldy_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.y.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.y == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by LDY
    }

    fn set_ldsp_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.sp.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.sp == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by LDSP
    }

    fn set_anda_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, self.reg.a.bit(7));
        self.reg.cc.set(CCFlag::Z, self.reg.a == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by ANDA
    }

    fn set_asl_flags(&mut self, new_val: u8, c: bool, v: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::C, c);
        self.reg.cc.set(CCFlag::V, v);
    }

    fn set_asr_flags(&mut self, new_val: u8, c: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::C, c);
        self.reg.cc.disable(CCFlag::V);
    }

    fn set_bita_flags(&mut self, result: u8) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by BITA
    }

    fn set_clr_flags(&mut self) {
        self.reg.cc.set(CCFlag::N, false);
        self.reg.cc.set(CCFlag::Z, true);
        self.reg.cc.set(CCFlag::V, false);
        self.reg.cc.set(CCFlag::C, false);
    }

    fn set_com_flags(&mut self, result: u8) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.set(CCFlag::V, false);
        // C is unaffected by COM
    }

    fn set_cmp_flags(&mut self, diff: u8, c: bool, v: bool) {
        self.reg.cc.set(CCFlag::N, diff.bit(7));
        self.reg.cc.set(CCFlag::Z, diff == 0);
        self.reg.cc.set(CCFlag::C, c);
        self.reg.cc.set(CCFlag::V, v);
    }

    fn set_dec_flags(&mut self, new_val: u8, v: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::V, v);
        // C is unaffected by DEC
    }

    fn set_inc_flags(&mut self, new_val: u8, v: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::V, v);
        // C is unaffected by INC
    }

    fn set_lsr_flags(&mut self, new_val: u8, c: bool, v: bool) {
        self.reg.cc.disable(CCFlag::N);
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::V, v);
        self.reg.cc.set(CCFlag::C, c);
    }

    fn set_neg_flags(&mut self, new_val: u8, old_val: u8, v: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::V, v);
        self.reg.cc.set(CCFlag::C, old_val != 0);
    }

    fn set_ora_flags(&mut self, result: u8) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.disable(CCFlag::V);
        // C is unaffected by ORA
    }

    fn set_rol_flags(&mut self, new_val: u8, c: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::V, new_val.bit(6) != new_val.bit(7));
        self.reg.cc.set(CCFlag::C, c);
    }

    fn set_ror_flags(&mut self, new_val: u8, c: bool) {
        self.reg.cc.set(CCFlag::N, new_val.bit(7));
        self.reg.cc.set(CCFlag::Z, new_val == 0);
        self.reg.cc.set(CCFlag::V, new_val.bit(6) != new_val.bit(7));
        self.reg.cc.set(CCFlag::C, c);
    }

    fn set_sbc_flags(&mut self, result: u8, c: bool, v: bool) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.set(CCFlag::C, c);
        self.reg.cc.set(CCFlag::V, v);
    }

    fn set_tst_flags(&mut self, result: u8) {
        self.reg.cc.set(CCFlag::N, result.bit(7));
        self.reg.cc.set(CCFlag::Z, result == 0);
        self.reg.cc.disable(CCFlag::V);
        self.reg.cc.disable(CCFlag::C);
    }

    fn todo(&mut self, instruction: u8) {
        self.debug_log(format!("Not yet implemented: {:02x}", instruction));
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
