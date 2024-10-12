use crate::{assets::{Font, GraphicNum}, def::UserState, vga_render::VGARenderer, vh::{WHITE, draw_tile_8}};

pub fn print(rdr: &VGARenderer, user_state: &mut UserState, str: &str, px_in: usize, py_in: usize) {
    let font = &rdr.fonts[user_state.font_number];
    let lines = str.split("\n");
    let mut px = px_in;
    let mut py = py_in;
    for line in lines {
        let (_, h) = measure_string(font, line);
        draw_string(rdr, font, line, px, py, user_state.font_color);
        px = user_state.window_x;
        py += h;
    }
}

/// Prints a string centered in the current window.
pub fn print_centered(rdr: &VGARenderer, user_state: &mut UserState, str: &str) {
    let font = &rdr.fonts[user_state.font_number];
    let (w, h) = measure_string(font, str);
    let px = user_state.window_x + ((user_state.window_w - w) / 2);
    let py = user_state.window_y + ((user_state.window_h - h) / 2);
    draw_string(rdr, font, str, px, py, user_state.font_color);
}

fn draw_string(rdr: &VGARenderer, font: &Font, str: &str, px_in: usize, py: usize, color: u8) {
    let mut px = px_in;
    for c in str.chars() {
        if let Some(ascii) = c.as_ascii() {
            let width = font.width[ascii as usize] as usize;
            for y in 0..font.height as usize {
                for x in 0..width {
                    let pix_data = font.data[ascii as usize][y*width+x];
                    if pix_data != 0 {
                        rdr.plot(px + x, py + y, color)
                    } 
                }
            }
            px += width;
        }
    }
}

/// Returns a (width, height) tupel.
fn measure_string(font: &Font, str: &str) -> (usize, usize) {
    let mut w : usize = 0;
    for c in str.chars() {
        if let Some(ascii) = c.as_ascii() {
            w += font.width[ascii as usize] as usize;
        }
    }
    return (w, font.height as usize);
}

pub fn draw_window(rdr: &VGARenderer, user_state: &mut UserState, x: usize, y: usize, width: usize, height: usize) {
    user_state.window_x = x * 8;
    user_state.window_y = y * 8;
    user_state.window_w = width * 8;
    user_state.window_h = height * 8;

    clear_window(rdr, user_state);

    let sx = (x-1)*8;
    let sy = (y-1)*8;
    let sw = (width+1)*8;
    let sh = (height+1)*8;

    draw_tile_8(rdr, sx, sy, 0);
    draw_tile_8(rdr, sx, sy + sh, 5);
    let mut i = sx + 8;
    while i <= sx + sw - 8 {
        draw_tile_8(rdr, i, sy, 1);
        draw_tile_8(rdr, i, sy + sh, 6);   
        i += 8;
    }
    draw_tile_8(rdr, i, sy, 2);
    draw_tile_8(rdr, i, sy + sh, 7);
    i = sy + 8;
    while i <= sy + sh - 8 {
        draw_tile_8(rdr, sx, i, 3);
        draw_tile_8(rdr, sx + sw, i, 4);
        i += 8;
    }
}

pub fn clear_window(rdr: &VGARenderer, user_state: &mut UserState) {
    rdr.bar(user_state.window_x, user_state.window_y, user_state.window_w, user_state.window_h, WHITE);
}