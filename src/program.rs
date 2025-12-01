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

pub struct Program {
    memory: [Register; 256],
    a: Register,
    x: Register,
    y: Register,
    sp: Register,
    pc: Register,
    ta: Register,
    cc: Register,
    ld: Register,
    exit: bool,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            memory: [Register::default(); 256],
            a: Default::default(),
            x: Default::default(),
            y: Default::default(),
            sp: Default::default(),
            pc: Default::default(),
            ta: Default::default(),
            cc: Default::default(),
            ld: Default::default(),
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

    pub fn reg_a(&self) -> &Register {
        &self.a
    }
    pub fn reg_x(&self) -> &Register {
        &self.x
    }
    pub fn reg_y(&self) -> &Register {
        &self.y
    }
    pub fn reg_sp(&self) -> &Register {
        &self.sp
    }
    pub fn reg_pc(&self) -> &Register {
        &self.pc
    }
    pub fn reg_ta(&self) -> &Register {
        &self.ta
    }
    pub fn reg_cc(&self) -> &Register {
        &self.cc
    }
    pub fn reg_ld(&self) -> &Register {
        &self.ld
    }

    pub fn execute(&mut self) {
        while !self.exit {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.pc.set(self.pc.get().overflowing_add(1).0);
    }
}
