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
    cc: Register,
    ld: Register,
}

pub struct Program {
    memory: [Register; 256],
    registers: RegisterStore,
    q_state: u8,
    exit: bool,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            memory: [Register::default(); 256],
            registers: RegisterStore::default(),
            q_state: 0,
            exit: false,
        }
    }
}

impl Program {
    pub fn load_memory(&mut self, data: &[u8; 256]) {
        for i in 0..256 {
            self.memory[i] = Register::new(data[i]);
        }
    }

    pub fn memory(&self) -> &[Register; 256] {
        &self.memory
    }

    pub fn memory_at(&self, addr: u8) -> u8 {
        self.memory[addr as usize].get()
    }

    pub fn reg_a(&self) -> Register {
        self.registers.a
    }
    pub fn reg_x(&self) -> Register {
        self.registers.x
    }
    pub fn reg_y(&self) -> Register {
        self.registers.y
    }
    pub fn reg_r(&self) -> Register {
        self.registers.r
    }
    pub fn reg_sp(&self) -> Register {
        self.registers.sp
    }
    pub fn reg_pc(&self) -> Register {
        self.registers.pc
    }
    pub fn reg_ta(&self) -> Register {
        self.registers.ta
    }
    pub fn reg_cc(&self) -> Register {
        self.registers.cc
    }
    pub fn reg_ld(&self) -> Register {
        self.registers.ld
    }

    pub fn execute(&mut self) {
        while !self.exit {
            self.step();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn step(&mut self) {
        match self.q_state {
            0 => self.registers.pc.set(self.memory_at(0xff)),
            _ => {}
        }

        self.q_state = self.q_state.overflowing_add(1).0;
        self.registers
            .pc
            .set(self.registers.pc.get().overflowing_add(1).0);

        if self.registers.pc.get() >= 100 {
            self.exit();
        }
    }
}
