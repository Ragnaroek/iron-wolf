#[cfg(test)]
#[path = "./fixed_test.rs"]
mod fixed_test;

use std::fmt;

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct Fixed(i32); //16:16 fixed point

pub fn new_fixed_u16(int_part: u16, frac_part: u16) -> Fixed {
    new_fixed_u32((int_part as u32) << 16 | frac_part as u32)
}

pub fn new_fixed_i16(int_part: i16, frac_part: i16) -> Fixed {
    new_fixed(int_part as i32, frac_part as i32)
}

pub fn new_fixed(int_part: i32, frac_part: i32) -> Fixed {
    Fixed(int_part << 16 | frac_part)
}

pub fn new_fixed_i32(raw: i32) -> Fixed {
    Fixed(raw)
}

pub fn new_fixed_u32(raw: u32) -> Fixed {
    Fixed(raw as i32)
}

impl Fixed {
    pub fn to_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0 >> 16, self.0 & 0xFFFF)
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i = self.0 >> 16;
        let frac = self.0 & 0xFFFF;
        write!(f, "{:#04x}.{:#04x}({}.{})", i, frac, i, frac)
    }
}

impl std::ops::Neg for Fixed {
    type Output = Self;

    fn neg(self) -> Self::Output {
        new_fixed_i32(-self.0)
    }
}

pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    new_fixed_i32(((a.to_i32() as i64 * b.to_i32() as i64) + 0x8000 >> 16) as i32)
}

pub fn fixed_by_frac(a_f: Fixed, b_f: Fixed) -> Fixed {
    let a = a_f.to_i32();
    let b = b_f.to_i32();
    let bx = (b & 0xFFFF) as i16;
    let mut si = ((b >> 16) & 0xFFFF) as i16;
    let mut ax = (a & 0xFFFF) as i16;
    let mut cx = ((a >> 16) & 0xFFFF) as i16;

    if cx < 0 {
        (cx, _) = cx.overflowing_neg();
        let cf = if ax == 0 { 0 } else { 1 };
        (ax, _) = ax.overflowing_neg();
        (cx, _) = cx.overflowing_sub(cf);
        si = (si as u16 ^ 0x8000) as i16; // toggle sign of result
    }

    let (dx, _) = mul(ax, bx); // fraction * fraction
    let di = dx;
    let ax = cx;
    let (mut dx, ax) = mul(ax, bx); // units * fraction

    let (ax_unsigned, cf) = (ax as u16).overflowing_add(di as u16);
    let mut ax = ax_unsigned as i16;
    if cf {
        dx += 1
    }

    if si as u16 & 0x8000 != 0 {
        (dx, _) = dx.overflowing_neg();
        let cf = if ax == 0 { 0 } else { 1 };
        (ax, _) = ax.overflowing_neg();
        dx = dx - cf;
    }

    new_fixed_u16(dx as u16, ax as u16)
}

fn mul(a: i16, b: i16) -> (i16, i16) {
    let (wa, _) = (a as i32 & 0xFFFF).overflowing_mul(b as i32 & 0xFFFF);
    (((wa >> 16) & 0xFFFF) as i16, (wa & 0xFFFF) as i16)
}
