#[cfg(test)]
#[path = "./draw_test.rs"]
mod draw_test;

use super::def::{Fixed, new_fixed_u16, new_fixed_i32, MIN_DIST};

pub fn fixed_by_frac(a_f: Fixed, b_f: Fixed) -> Fixed {
    let a = a_f.to_i32();
    let b = b_f.to_i32();
    let bx = (b & 0xFFFF) as i16;
    let mut si = (((b >> 16) & 0xFFFF)) as i16;
    let mut ax = (a & 0xFFFF) as i16;
    let mut cx = ((a >> 16) & 0xFFFF) as i16;

    if cx < 0 {
        (cx, _) = cx.overflowing_neg();
        let cf = if ax == 0 {0} else {1};
        (ax, _) = ax.overflowing_neg();
        (cx, _) = cx.overflowing_sub(cf);
        si = (si as u16 ^ 0x8000) as i16; // toggle sign of result
    } 

    let (dx, mut ax) = mul(ax, bx); // fraction * fraction
    let di = dx;
    ax = cx;
    let (mut dx, ax) = mul(ax, bx); // units * fraction

    let (ax_unsigned, cf) = (ax as u16).overflowing_add(di as u16);
    let mut ax = ax_unsigned as i16;
    if cf {
        dx += 1
    }

    if si as u16 & 0x8000 != 0 {
        (dx, _) = dx.overflowing_neg();
        let cf = if ax == 0 {0} else {1};
        (ax, _) = ax.overflowing_neg();
        dx = dx - cf;
    }

    new_fixed_u16(dx as u16, ax as u16)
}

fn mul(a: i16, b: i16) -> (i16, i16) {
    let (wa, _) = (a as i32 & 0xFFFF).overflowing_mul(b as i32 & 0xFFFF);
    (((wa >> 16) & 0xFFFF) as i16, (wa & 0xFFFF) as i16)
}

pub fn calc_height(height_numerator: i32, x_intercept: i32, y_intercept: i32, view_x: i32, view_y: i32, view_cos: Fixed, view_sin: Fixed) -> i32 {
    let gx = new_fixed_i32(x_intercept - view_x);
    let gxt = fixed_by_frac(gx, view_cos);

    let gy = new_fixed_i32(y_intercept - view_y);
    let gyt = fixed_by_frac(gy, view_sin);

    let mut nx = gxt.to_i32() - gyt.to_i32();

    if nx < MIN_DIST {
         nx = MIN_DIST;
    }

    height_numerator/(nx >> 8)
}