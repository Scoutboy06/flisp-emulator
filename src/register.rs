use std::ops;

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Register {
    data: u8,
}

impl Register {
    pub fn new(data: u8) -> Self {
        Self { data }
    }

    pub fn set(&mut self, data: impl Into<u8>) {
        self.data = data.into();
    }

    pub fn get(&self) -> u8 {
        self.data
    }

    pub fn bit(&self, bit_idx: u8) -> bool {
        self.data & (1u8 << bit_idx) != 0
    }

    pub fn inc(&mut self) -> bool {
        let res = self.data.overflowing_add(1);
        self.data = res.0;
        res.1
    }

    pub fn dec(&mut self) -> bool {
        let res = self.data.overflowing_sub(1);
        self.data = res.0;
        res.1
    }
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        self.data
    }
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        Register::new(value)
    }
}

impl ops::Add for Register {
    type Output = (u8, bool);
    fn add(self, rhs: Self) -> Self::Output {
        self.data.overflowing_add(rhs.data)
    }
}

impl ops::Add<u8> for Register {
    type Output = (u8, bool);
    fn add(self, rhs: u8) -> Self::Output {
        self.data.overflowing_add(rhs)
    }
}

impl ops::Add<Register> for u8 {
    type Output = (u8, bool);
    fn add(self, rhs: Register) -> Self::Output {
        self.overflowing_add(rhs.data)
    }
}

impl ops::Sub for Register {
    type Output = (u8, bool);
    fn sub(self, rhs: Self) -> Self::Output {
        self.data.overflowing_sub(rhs.data)
    }
}

impl PartialEq<u8> for Register {
    fn eq(&self, other: &u8) -> bool {
        self.data == *other
    }
}

impl PartialEq<Register> for u8 {
    fn eq(&self, other: &Register) -> bool {
        *self == other.data
    }
}
