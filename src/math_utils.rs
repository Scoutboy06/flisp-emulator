pub trait GetBit {
    fn bit(&self, bit: u8) -> bool;
}

impl GetBit for u8 {
    fn bit(&self, bit_idx: u8) -> bool {
        *self & (1u8 << bit_idx) != 0
    }
}

pub fn add<T, K>(x: T, y: K) -> (u8, bool, bool)
where
    T: Into<u8>,
    K: Into<u8>,
{
    add_c(x, y, false)
}

/// 8-bit addition with optional carry-in
///
/// Returns: (sum, c_flag, v_flag)
///
/// Performs x + y + cin
pub fn add_c<T, K>(x: T, y: K, cin: bool) -> (u8, bool, bool)
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

/// 8-bit subtraction
///
/// Returns: (difference, c_flag, v_flag)
///
/// Performs x - y
pub fn sub<T, K>(x: T, y: K) -> (u8, bool, bool)
where
    T: Into<u8>,
    K: Into<u8>,
{
    sub_c(x, y, false)
}

/// 8-bit subtraction with optional carry-in
///
/// Returns: (difference, c_flag, v_flag)
///
/// Performs x - y - cin
pub fn sub_c<T, K>(x: T, y: K, cin: bool) -> (u8, bool, bool)
where
    T: Into<u8>,
    K: Into<u8>,
{
    let x = x.into();
    let y = y.into();
    let (diff1, c1) = x.overflowing_sub(y);
    let (diff2, c2) = diff1.overflowing_sub(cin as u8);

    let r7 = diff2.bit(7);
    let x7 = x.bit(7);
    let y7 = y.bit(7);
    let v = (r7 && !x7 && !y7) || (!r7 && x7 && y7);

    (diff2, c1 || c2, v)
}

/// Logical Shift Left
///
/// Returns: (result, c_flag, v_flag)
///
/// Performs (x << 1)
pub fn shl<T: Into<u8>>(x: T) -> (u8, bool, bool) {
    let x = x.into();
    let c = x.bit(7);
    let (res, _) = x.overflowing_shl(1);
    let v = c != res.bit(7);
    (res, c, v)
}

/// Logical Shift Right
///
/// Returns: (result, c_flag, v_flag)
///
/// Performs (x >> 1)
pub fn shr<T: Into<u8>>(x: T) -> (u8, bool, bool) {
    let x = x.into();
    let c = x.bit(0);
    let res = x >> 1;
    let v = res.bit(7) != x.bit(7);
    (res, c, v)
}

/// Arithmetic Shift Right
///
/// Returns: (result, c_flag)
///
/// Performs (x >> 1) with sign bit preserved
pub fn shr_signed<T: Into<u8>>(x: T) -> (u8, bool) {
    let x = x.into();
    let c = x.bit(0);
    let sign_bit = x.bit(7);
    let res = (x >> 1) | ((sign_bit as u8) << 7);
    (res, c)
}

/// Rotate Left
///
/// Returns: (result, c_flag)
///
/// Performs (x << 1) with bit 7 wrapped to bit 0
pub fn rotate_left<T: Into<u8>>(x: T) -> (u8, bool) {
    let x = x.into();
    let c = x.bit(7);
    let res = (x << 1) | (c as u8);
    (res, c)
}

/// Rotate Right
///
/// Returns: (result, c_flag)
///
/// Performs (x >> 1) with bit 0 wrapped to bit 7
pub fn rotate_right<T: Into<u8>>(x: T) -> (u8, bool) {
    let x = x.into();
    let c = x.bit(0);
    let res = (x >> 1) | ((c as u8) << 7);
    (res, c)
}
