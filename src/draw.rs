#[cfg(test)]
#[path = "./draw_test.rs"]
mod draw_test;

use libiw::gamedata::Texture;

use super::def::{Fixed, Assets, new_fixed_u16, new_fixed_i32, MIN_DIST};
use super::play::RayCast;
use super::vga_render::{Renderer, SCREENBWIDE};

pub struct ScalerState {
    last_side: bool,
    post_x: usize,
    post_width: usize,
    texture_ix: usize,
}

pub fn initial_scaler_state() -> ScalerState {
    ScalerState { 
        last_side: false,
        post_x: 0,
        post_width: 1,
        texture_ix: 0,
     }
}

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

pub fn scale_post(scaler_state: &ScalerState, height: i32, view_height: usize, rdr: &dyn Renderer, assets: &Assets) {
    let texture = &assets.textures[scaler_state.texture_ix];

    // TODO lookup "compiled" scaler here
    //full_scale(height, view_height, texture, rdr)
}

/*
fn draw_scaled(x: usize, post_src: i32, height: i32, view_height: i32, texture: Option<&Texture>, rdr: &dyn Renderer) {
    //TODO use the exact copy statements as the compiled scalers do! (compare scaling code in the original with this => step_size and clamping)
    let line_height = if height > 512 {
        view_height
    } else {
        (height as f64 / 512.0 * view_height as f64) as i32
    };
    let step = TEXTURE_HEIGHT as f64 / line_height as f64;
   
    let y = view_height/2 - line_height/2;

    let mut src = post_src as f64;
    for y_draw in y..(y+line_height) {
        let pixel = if let Some(tex) = texture {
            tex.bytes[src as usize]
        } else {
            0x50
        };
        // TODO replace this with a faster? buffered draw
        rdr.plot(x, y_draw as usize, pixel);
        src += step;
    }
}*/

pub fn hit_vert_wall(scaler_state : &mut ScalerState, rc : &RayCast, pixx: usize, height: i32, view_height: usize, rdr: &dyn Renderer, assets: &Assets) {
    let post_source = 0xFC0 - ((rc.x_intercept>>4) & 0xFC0);

    if scaler_state.last_side {
        scale_post(scaler_state, height, view_height, rdr, assets)
    }
    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.texture_ix = 49; //TODO only for testing!
}

pub fn hit_horiz_wall(scaler_state : &mut ScalerState, rc : &RayCast, pixx: usize, height: i32) {

}

pub fn hit_horiz_door() {

}

pub fn hit_vert_door() {

}

pub fn hit_horiz_pwall() {

}

pub fn hit_vert_pwall() {

}