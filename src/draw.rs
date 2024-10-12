#[cfg(test)]
#[path = "./draw_test.rs"]
mod draw_test;

use libiw::gamedata::Texture;

use crate::def::TILEGLOBAL;
use crate::play::ProjectionConfig;

use super::def::{Fixed, Assets, new_fixed_u16, new_fixed_i32, MIN_DIST};
use super::play::RayCast;
use super::vga_render::{Renderer, SCREENBWIDE};

static MAP_MASKS_1 : [u8; 4*8] = [
    1 ,3 ,7 ,15,15,15,15,15,
    2 ,6 ,14,14,14,14,14,14,
    4 ,12,12,12,12,12,12,12,
    8 ,8 ,8 ,8 ,8 ,8 ,8 ,8
];

pub struct ScalerState {
    last_side: bool,
    post_x: usize,
    post_width: usize,
    post_source: usize,
    texture_ix: usize,
}

pub fn initial_scaler_state() -> ScalerState {
    ScalerState { 
        last_side: false,
        post_x: 0,
        post_width: 1,
        post_source: 0,
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

pub fn scale_post(scaler_state: &ScalerState, height: i32, prj: &ProjectionConfig, rdr: &dyn Renderer, assets: &Assets) {
    let texture = &assets.textures[scaler_state.texture_ix];

    let mut h = ((height & 0xFFF8)>>1) as usize;
    if h > prj.scaler.max_scale_shl2 {
        h = prj.scaler.max_scale_shl2
    }

    //both additionally shift by 1, in the original the computed offsets are in 16-bit words that
    //point into a 32-bit array 
    let ix = prj.scaler.scale_call[h>>1];
    let scaler = &prj.scaler.scalers[ix>>1];

    let bx = ((scaler_state.post_x &0x03) << 3) + scaler_state.post_width;				
    rdr.set_mask(MAP_MASKS_1[bx-1]);

    let line_start = (scaler_state.post_x >> 2) + rdr.buffer_offset();
    for pix_scaler in &scaler.pixel_scalers {
        let pix = texture.bytes[scaler_state.post_source + pix_scaler.texture_src];
        for mem_dest in &pix_scaler.mem_dests {
            rdr.write_mem(line_start + *mem_dest as usize, pix);
        }
    }
}

pub fn hit_vert_wall(scaler_state : &mut ScalerState, rc : &mut RayCast, pixx: usize, height: i32, prj: &ProjectionConfig, rdr: &dyn Renderer, assets: &Assets) {
    let mut post_source = 0xFC0 - ((rc.y_intercept>>4) & 0xFC0);
    if rc.x_tilestep == -1 {
        post_source = 0xFC0-post_source;
        rc.x_intercept += TILEGLOBAL;
    }

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    let texture_ix = if rc.tile_hit & 0x040 != 0 {
        vert_wall(rc.tile_hit as usize & !0x40)
    } else {
        vert_wall(rc.tile_hit as usize)
    };

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

pub fn hit_horiz_wall(scaler_state : &mut ScalerState, rc : &mut RayCast, pixx: usize, height: i32, prj: &ProjectionConfig, rdr: &dyn Renderer, assets: &Assets) {
    let mut post_source = 0xFC0 - ((rc.x_intercept>>4) & 0xFC0);
    if rc.y_tilestep == -1 {
        rc.y_intercept += TILEGLOBAL;
    } else {
        post_source = 0xFC0-post_source;
    }

    if scaler_state.last_side {
        scale_post(scaler_state, height, prj, rdr, assets);
    }

    let texture_ix = if rc.tile_hit & 0x040 != 0 {
        horiz_wall(rc.tile_hit as usize & !0x40)
    } else {
        horiz_wall(rc.tile_hit as usize)
    };

    scaler_state.last_side = true;
    scaler_state.post_x = pixx;
    scaler_state.post_width = 1;
    scaler_state.post_source = post_source as usize;
    scaler_state.texture_ix = texture_ix;
}

fn horiz_wall(i: usize) -> usize {
    (i-1)*2
}

fn vert_wall(i: usize) -> usize {
    (i-1)*2+1
}

pub fn hit_horiz_door() {
}

pub fn hit_vert_door() {
}

pub fn hit_horiz_pwall() {
}

pub fn hit_vert_pwall() {
}