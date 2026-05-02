// Float64 configuration constants
const DOUBLE_MANTISSA_BITS: u32 = 52;
const DOUBLE_BIAS: u32 = 1023;
const DOUBLE_POW5_INV_BITCOUNT: u32 = 125;
const DOUBLE_POW5_BITCOUNT: u32 = 125;

use crate::ryu::ryu_math::*;
use crate::ryu::tables_f64::*;

pub fn float64_to_string(d: f64) -> String {
    let bits = d.to_bits();
    
    let sign = (bits >> 63) != 0;
    let ieee_mantissa = bits & 0xFFFFFFFFFFFFF;
    let ieee_exponent = ((bits >> 52) & 0x7FF) as u32;
    
    if ieee_exponent == 2047 {
        if ieee_mantissa == 0 {
            return if sign { "-Infinity".to_string() } else { "Infinity".to_string() };
        }
        return "NaN".to_string();
    }
    if ieee_exponent == 0 && ieee_mantissa == 0 {
        return if sign { "-0E0".to_string() } else { "0E0".to_string() };
    }
    
    let e2 = if ieee_exponent == 0 {
        1 - DOUBLE_BIAS as i32 - DOUBLE_MANTISSA_BITS as i32 - 2
    } else {
        ieee_exponent as i32 - DOUBLE_BIAS as i32 - DOUBLE_MANTISSA_BITS as i32 - 2
    };
    
    let m2 = if ieee_exponent == 0 {
        ieee_mantissa
    } else {
        (1u64 << DOUBLE_MANTISSA_BITS) | ieee_mantissa
    };
    
    let even = (m2 & 1) == 0;
    let accept_bounds = even;
    
    let mv = m2 * 4;
    let mp = mv + 2;
    let mm_shift = if ieee_mantissa != 0 || ieee_exponent <= 1 { 1u64 } else { 0u64 };
    let mm = mv - 1 - mm_shift;
    
    let mut vr_is_trailing_zeros = false;
    let mut vm_is_trailing_zeros = false;
    let mut last_digit = 0u64;
    let e10: i32;
    let vr: u64;
    let mut vp: u64;
    let vm_: u64;
    
    if e2 >= 0 {
        let q = log10_pow2(e2);
        e10 = q as i32;
        let k = DOUBLE_POW5_INV_BITCOUNT + pow5bits(q) - 1;
        let i = -e2 + q as i32 + k as i32;
        
        vr = mul_shift_64(mv, &DOUBLE_POW5_INV_SPLIT[(q as i32) as usize], i as u32);
        vp = mul_shift_64(mp, &DOUBLE_POW5_INV_SPLIT[(q as i32) as usize], i as u32);
        vm_ = mul_shift_64(mm, &DOUBLE_POW5_INV_SPLIT[(q as i32) as usize], i as u32);
        
        if q != 0 && (vp - 1) / 10 <= vm_ / 10 {
            let l = DOUBLE_POW5_INV_BITCOUNT + pow5bits(q - 1) - 1;
            last_digit = mul_shift_64(mv, &DOUBLE_POW5_INV_SPLIT[((q - 1) as i32) as usize], (-e2 + q as i32 - 1 + l as i32) as u32) % 10;
        }
        
        if q <= 21 {
            if mv % 5 == 0 {
                vr_is_trailing_zeros = multiple_of_power_of_5_64(mv, q);
            } else if accept_bounds {
                vm_is_trailing_zeros = multiple_of_power_of_5_64(mm, q);
            } else {
                if multiple_of_power_of_5_64(mp, q) {
                    vp -= 1;
                }
            }
        }
    } else {
        let q = log10_pow5(-e2);
        e10 = q as i32 + e2;
        let i = -e2 - q as i32;
        let k = pow5bits(i) - DOUBLE_POW5_BITCOUNT;
        let j = q as i32 - k as i32;
        
        vr = mul_shift_64(mv, &DOUBLE_POW5_SPLIT[i as usize], j as u32);
        vp = mul_shift_64(mp, &DOUBLE_POW5_SPLIT[i as usize], j as u32);
        vm_ = mul_shift_64(mm, &DOUBLE_POW5_SPLIT[i as usize], j as u32);
        
        if q != 0 && (vp - 1) / 10 <= vm_ / 10 {
            let j2 = q as i32 - 1 - (pow5bits(i + 1) - DOUBLE_POW5_BITCOUNT) as i32;
            last_digit = mul_shift_64(mv, &DOUBLE_POW5_SPLIT[(i + 1) as usize], j2 as u32) % 10;
        }
        
        if q <= 1 {
            vr_is_trailing_zeros = true;
            if accept_bounds {
                vm_is_trailing_zeros = mm_shift == 1;
            } else {
                vp -= 1;
            }
        } else if q < 63 {
            vr_is_trailing_zeros = multiple_of_power_of_2_64(mv, q - 1);
            if accept_bounds {
                vm_is_trailing_zeros = multiple_of_power_of_5_64(mm, q);
            } else {
                if multiple_of_power_of_5_64(mp, q) {
                    vp -= 1;
                }
            }
        }
    }
    
    let mut removed = 0;
    let mut vr2 = vr;
    let mut vp2 = vp;
    let mut vm2 = vm_;
    
    if vm_is_trailing_zeros || vr_is_trailing_zeros {
        while vp2 / 10 > vm2 / 10 {
            vm_is_trailing_zeros = vm_is_trailing_zeros && (vm2 % 10 == 0);
            vr_is_trailing_zeros = vr_is_trailing_zeros && (last_digit == 0);
            last_digit = vr2 % 10;
            vr2 /= 10;
            vp2 /= 10;
            vm2 /= 10;
            removed += 1;
        }
        
        if vm_is_trailing_zeros {
            while vm2 % 10 == 0 {
                vr_is_trailing_zeros = vr_is_trailing_zeros && (last_digit == 0);
                last_digit = vr2 % 10;
                vr2 /= 10;
                vp2 /= 10;
                vm2 /= 10;
                removed += 1;
            }
        }
        
        if vr_is_trailing_zeros && last_digit == 5 && (vr2 & 1) == 0 {
            last_digit = 4;
        }
        
        let round_up = (vr2 == vm2 && (!accept_bounds || !vm_is_trailing_zeros)) || last_digit >= 5;
        let output = if round_up { vr2 + 1 } else { vr2 };
        let exp = e10 + removed;
        let olength = decimal_length17(output);
        
        let mut result = String::new();
        if sign { result.push('-'); }
        let digits = output.to_string();
        if olength == 1 {
            result.push_str(&digits);
        } else {
            result.push_str(&format!("{}.{}", &digits[0..1], &digits[1..]));
        }
        result.push_str(&format!("E{}", exp + olength as i32 - 1));
        return result;
    } else {
        while vp2 / 10 > vm2 / 10 {
            last_digit = vr2 % 10;
            vr2 /= 10;
            vp2 /= 10;
            vm2 /= 10;
            removed += 1;
        }
        
        let output = if vr2 == vm2 || last_digit >= 5 { vr2 + 1 } else { vr2 };
        let exp = e10 + removed;
        let olength = decimal_length17(output);
        
        let mut result = String::new();
        if sign { result.push('-'); }
        let digits = output.to_string();
        if olength == 1 {
            result.push_str(&digits);
        } else {
            result.push_str(&format!("{}.{}", &digits[0..1], &digits[1..]));
        }
        result.push_str(&format!("E{}", exp + olength as i32 - 1));
        return result;
    }
}