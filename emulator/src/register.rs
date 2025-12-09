use std::ops;

use crate::math_utils::{add_c, sub};

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

    /// 8-bit increment
    ///
    /// Performs `self = self + 1` with overflow handling
    ///
    /// Returns: (`c_flag`, `v_flag`)
    pub fn inc(&mut self) -> (bool, bool) {
        let res = add_c(self.data, 1, false);
        self.data = res.0;
        (res.1, res.2)
    }

    /// 8-bit decrement
    ///
    /// Performs `self = self - 1` with overflow handling
    ///
    /// Returns: (`c_flag`, `v_flag`)
    pub fn dec(&mut self) -> (bool, bool) {
        let res = sub(self.data, 1);
        self.data = res.0;
        (res.1, res.2)
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
    type Output = (u8, bool, bool);
    /// 8-bit addition
    /// Returns: (sum, c_flag, v_flag)
    fn add(self, rhs: Self) -> Self::Output {
        add_c(self.data, rhs.data, false)
    }
}

impl ops::Add<u8> for Register {
    type Output = (u8, bool, bool);
    /// 8-bit addition
    /// Returns: (sum, c_flag, v_flag)
    fn add(self, rhs: u8) -> Self::Output {
        add_c(self.data, rhs, false)
    }
}

impl ops::Add<Register> for u8 {
    type Output = (u8, bool, bool);
    /// 8-bit addition
    /// Returns: (sum, c_flag, v_flag)
    fn add(self, rhs: Register) -> Self::Output {
        add_c(self, rhs.data, false)
    }
}

impl ops::BitAnd for Register {
    type Output = u8;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.data & rhs.data
    }
}

impl ops::BitAnd<u8> for Register {
    type Output = u8;
    fn bitand(self, rhs: u8) -> Self::Output {
        self.data & rhs
    }
}

impl ops::BitAnd<Register> for u8 {
    type Output = u8;
    fn bitand(self, rhs: Register) -> Self::Output {
        self & rhs.data
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
