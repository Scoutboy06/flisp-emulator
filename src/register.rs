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

    pub fn inc(&mut self) -> (bool, bool) {
        let res = add(self.data, 1, false);
        self.data = res.0;
        (res.1, res.2)
    }

    /// Decrement register by 1
    /// Returns: (c_flag, v_flag)
    pub fn dec(&mut self) -> (bool, bool) {
        let res = sub(self.data, 1);
        self.data = res.0;
        (res.1, res.2)
    }

    /// 8-bit addition with carry-in
    /// Returns: (sum, c_flag, v_flag)
    pub fn add_c<T: Into<Self>>(&self, other: T) -> (u8, bool, bool) {
        add(self.data, other.into().data, true)
    }
}

/// 8-bit addition with optional carry-in
/// Returns: (sum, c_flag, v_flag)
pub fn add<T, K>(x: T, y: K, cin: bool) -> (u8, bool, bool)
where
    T: Into<u8>,
    K: Into<u8>,
{
    let x = x.into();
    let y = y.into();
    let (sum1, c1) = x.overflowing_add(y);
    let (sum2, c2) = sum1.overflowing_add(cin as u8);

    let r7 = sum2.bit(7);
    let x7 = x.bit(7);
    let y7 = y.bit(7);
    let v = (r7 && !x7 && !y7) || (!r7 && x7 && y7);

    (sum2, c1 || c2, v)
}

pub fn sub<T, K>(x: T, y: K) -> (u8, bool, bool)
where
    T: Into<u8>,
    K: Into<u8>,
{
    let x = x.into();
    let y = y.into();
    let (diff, c) = x.overflowing_sub(y);

    let r7 = diff.bit(7);
    let x7 = x.bit(7);
    let y7 = y.bit(7);
    let v = (r7 && !x7 && y7) || (!r7 && x7 && !y7);

    (diff, c, v)
}

/// Logical Shift Left
/// (x << 1)
/// Returns: (result, c_flag, v_flag)
pub fn shl<T: Into<u8>>(x: T) -> (u8, bool, bool) {
    let x = x.into();
    let c = x.bit(7);
    let (res, _) = x.overflowing_shl(1);
    let v = c != res.bit(7);
    (res, c, v)
}

/// Logical Shift Right
/// (x >> 1)
/// Returns: (result, c_flag, v_flag)
pub fn shr<T: Into<u8>>(x: T) -> (u8, bool, bool) {
    let x = x.into();
    let c = x.bit(0);
    let res = x >> 1;
    let v = res.bit(7) != x.bit(7);
    (res, c, v)
}

/// Arithmetic Shift Right
/// (x >> 1) with sign bit preserved
/// Returns: (result, c_flag)
pub fn shr_signed<T: Into<u8>>(x: T) -> (u8, bool) {
    let x = x.into();
    let c = x.bit(0);
    let sign_bit = x.bit(7);
    let res = (x >> 1) | ((sign_bit as u8) << 7);
    (res, c)
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
        add(self.data, rhs.data, false)
    }
}

impl ops::Add<u8> for Register {
    type Output = (u8, bool, bool);
    /// 8-bit addition
    /// Returns: (sum, c_flag, v_flag)
    fn add(self, rhs: u8) -> Self::Output {
        add(self.data, rhs, false)
    }
}

impl ops::Add<Register> for u8 {
    type Output = (u8, bool, bool);
    /// 8-bit addition
    /// Returns: (sum, c_flag, v_flag)
    fn add(self, rhs: Register) -> Self::Output {
        add(self, rhs.data, false)
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

pub trait GetBit {
    fn bit(&self, bit: u8) -> bool;
}

impl GetBit for u8 {
    fn bit(&self, bit_idx: u8) -> bool {
        *self & (1u8 << bit_idx) != 0
    }
}
