
use std::ascii;
use crate::{vga_render::VGARenderer, def::UserState, vh::{vw_hlin, vw_vlin}, us1::print};

const STRIPE : u8 = 0x2c;
const BORDER_COLOR : u8 = 0x29;
const BORDER2_COLOR : u8 = 0x23;
const DEACTIVATE : u8 = 0x2b;

const TEXT_COLOR : u8 = 0x17;
const HIGHLIGHT : u8 = 0x13;

pub fn draw_stripes(rdr: &VGARenderer, y: usize) {
    rdr.bar(0, y, 320, 24, 0);
    rdr.hlin(0, 319, y+22, STRIPE);
}

pub fn clear_ms_screen(rdr: &VGARenderer) {
    rdr.bar(0, 0, 320, 200, BORDER_COLOR)
}

/// The supplied message should only contain ASCII characters.
/// All other characters are not supported and ignored.
pub fn message(rdr: &VGARenderer, user_state: &mut UserState, str: &str) {
    user_state.font_number = 1;
    user_state.font_color = 0;
    let font = &rdr.fonts[user_state.font_number];
    let mut h = font.height as usize;
    let mut w : usize = 0;
    let mut mw : usize = 0;
    for c in str.chars() {
        if let Some(ascii_char) = c.as_ascii() {
            if ascii_char == ascii::Char::LineFeed {
                if w > mw {
                    mw = w;
                }
                w = 0;
                h += font.height as usize;
            } else {
                w += font.width[ascii_char as usize] as usize;
            }
        }
    }

    if w+10 > mw {
        mw = w + 10;
    }

    let print_y = (user_state.window_h/2)-h/2;
    user_state.window_x = 160-mw/2;
    
    let prev_buffer = rdr.buffer_offset();
    rdr.set_buffer_offset(rdr.active_buffer());
    draw_window(rdr, user_state.window_x-5, print_y-5, mw+10, h+10, TEXT_COLOR);
    draw_outline(rdr, user_state.window_x-5, print_y-5, mw+10, h+10, 0, HIGHLIGHT);
    print(rdr, user_state, str, user_state.window_x, print_y);
    rdr.set_buffer_offset(prev_buffer);
}

pub fn draw_window(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color: u8) {
    rdr.bar(x, y, width, height, color);
    draw_outline(rdr, x, y, width, height, BORDER2_COLOR, DEACTIVATE);
}

pub fn draw_outline(rdr: &VGARenderer, x: usize, y: usize, width: usize, height: usize, color1: u8, color2: u8) {
    vw_hlin(rdr, x, x+width, y, color2);
    vw_vlin(rdr, y, y+height, x, color2);
    vw_hlin(rdr, x,x+width,y+height, color1);
    vw_vlin(rdr, y, y+height, x+width, color1);
}