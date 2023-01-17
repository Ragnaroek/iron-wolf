pub fn fixed_mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) + 0x8000 >> 16) as i32
}