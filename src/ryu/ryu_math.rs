pub fn pow5bits(e: i32) -> i32 {
    e * 1217359 / 524288 + 1
}

pub fn log10_pow2(e: i32) -> i32 {
    e * 78913 / 262144
}

pub fn log10_pow5(e: i32) -> i32 {
    e * 732923 / 1048576
}

pub fn decimal_length9(v: u64) -> u32 {
    if v >= 100000000 { return 9; }
    if v >= 10000000 { return 8; }
    if v >= 1000000 { return 7; }
    if v >= 100000 { return 6; }
    if v >= 10000 { return 5; }
    if v >= 1000 { return 4; }
    if v >= 100 { return 3; }
    if v >= 10 { return 2; }
    return 1;
}

pub fn decimal_length17(v: u64) -> u32 {
    if v >= 10000000000000000 { return 17; }
    if v >= 1000000000000000 { return 16; }
    if v >= 100000000000000 { return 15; }
    if v >= 10000000000000 { return 14; }
    if v >= 1000000000000 { return 13; }
    if v >= 100000000000 { return 12; }
    if v >= 10000000000 { return 11; }
    if v >= 1000000000 { return 10; }
    if v >= 100000000 { return 9; }
    if v >= 10000000 { return 8; }
    if v >= 1000000 { return 7; }
    if v >= 100000 { return 6; }
    if v >= 10000 { return 5; }
    if v >= 1000 { return 4; }
    if v >= 100 { return 3; }
    if v >= 10 { return 2; }
    return 1;
}

pub fn mul_shift_32(m: u64, factor: u64, shift: u32) -> u64 {
    let factor_lo = factor & 0xFFFFFFFF;
    let factor_hi = factor >> 32;
    
    let bits0 = m * factor_lo;
    let bits1 = m * factor_hi;
    
    let sum_val = (bits0 >> 32) + bits1;
    (sum_val >> (shift - 32)) & 0xFFFFFFFF
}

pub fn mul_shift_64(m: u64, mul: &[u64], shift: u32) -> u64 {
    let b0 = (m as u128) * (mul[0] as u128);
    let b2 = (m as u128) * (mul[1] as u128);
    let b0_hi = b0 >> 64;
    let sum_val = b0_hi + b2;
    ((sum_val >> (shift - 64)) as u64) & 0xFFFFFFFFFFFFFFFF
}

pub fn multiple_of_power_of_5_64(value: u64, q: i32) -> bool {
    if q == 0 { return true; }
    if q >= 64 { return value == 0; }
    let pow5 = 5u64.pow(q as u32);
    (value % pow5) == 0
}

pub fn multiple_of_power_of_2_64(value: u64, q: i32) -> bool {
    if q == 0 { return true; }
    if q >= 64 { return value == 0; }
    (value & ((1 << q) - 1)) == 0
}