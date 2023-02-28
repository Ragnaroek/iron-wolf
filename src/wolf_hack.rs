use super::def::{Fixed, new_fixed_i32};


pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    new_fixed_i32(((a.to_i32() as i64 * b.to_i32() as i64) + 0x8000 >> 16) as i32)
}