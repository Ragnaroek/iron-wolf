use vga::input::NumCode;

use crate::assets::Font;
use crate::def::WindowState;
use crate::input::Input;
use crate::time::{Ticker, TICK_BASE};
use crate::vga_render::VGARenderer;
use crate::vh::{draw_tile_8, WHITE};

pub fn print(rdr: &VGARenderer, win_state: &mut WindowState, str: &str) {
    let font = &rdr.fonts[win_state.font_number];
    let lines: Vec<&str> = str.split("\n").collect();
    for i in 0..lines.len() {
        let line = lines[i];
        let (w, h) = measure_string(font, line);
        draw_string(rdr, font, line, win_state.print_x, win_state.print_y, win_state.font_color);
        
        if i == lines.len()-1 {
            win_state.print_x += w;
        } else {
            win_state.print_x = win_state.window_x;
            win_state.print_y += h;
        }
    }
}

/// Prints a string in the current window. Newlines are
/// supported.
pub fn c_print(rdr: &VGARenderer, win_state: &mut WindowState, str: &str) {
    let lines = str.split("\n");
    for line in lines {
        c_print_line(rdr, win_state, line);
    }
}

/// Prints a string centered on the current line and
/// advances to the next line. Newlines are not supported.
pub fn c_print_line(rdr: &VGARenderer, win_state: &mut WindowState, str: &str) {
    let font = &rdr.fonts[win_state.font_number];
    let (w, h) = measure_string(font, str);
    if w > win_state.window_w {
        panic!("c_print_line - String exceeds width")
    }
    let px = win_state.window_x + ((win_state.window_w - w) / 2);
    let py = win_state.print_y;
    draw_string(rdr, font, str, px, py, win_state.font_color);
    win_state.print_y += h;
}

/// Prints a string centered in the current window.
pub fn print_centered(rdr: &VGARenderer, win_state: &mut WindowState, str: &str) {
    let font = &rdr.fonts[win_state.font_number];
    let (w, h) = measure_string(font, str);
    let px = win_state.window_x + ((win_state.window_w - w) / 2);
    let py = win_state.window_y + ((win_state.window_h - h) / 2);
    draw_string(rdr, font, str, px, py, win_state.font_color);
}

fn draw_string(rdr: &VGARenderer, font: &Font, str: &str, px_in: usize, py: usize, color: u8) {
    let mut px = px_in;
    for c in str.chars() {
        let ext_ascii_val = c as usize;
        if ext_ascii_val <= 255 {
            let width = font.width[ext_ascii_val] as usize;
            for y in 0..font.height as usize {
                for x in 0..width {
                    let pix_data = font.data[ext_ascii_val][y*width+x];
                    if pix_data != 0 {
                        rdr.plot(px + x, py + y, color)
                    } 
                }
            }
            px += width;
        } else {
            panic!("non extended ascii char provided");
        }
    }
}

/// Returns a (width, height) tupel.
fn measure_string(font: &Font, str: &str) -> (usize, usize) {
    let mut w : usize = 0;
    for c in str.chars() {
        let ext_ascii_val = c as usize;
        if ext_ascii_val <= 255 {
            w += font.width[ext_ascii_val as usize] as usize;
        } else {
            panic!("non ext ascii char provided");
        }
    }
    return (w, font.height as usize);
}

pub fn draw_window(rdr: &VGARenderer, win_state: &mut WindowState, x: usize, y: usize, width: usize, height: usize) {
    win_state.window_x = x * 8;
    win_state.window_y = y * 8;
    win_state.window_w = width * 8;
    win_state.window_h = height * 8;
    win_state.print_x = win_state.window_x;
    win_state.print_y = win_state.window_y;    

    let sx = (x-1)*8;
    let sy = (y-1)*8;
    let sw = (width+1)*8;
    let sh = (height+1)*8;

    clear_window(rdr, win_state);

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

pub fn clear_window(rdr: &VGARenderer, win_state: &mut WindowState) {
    rdr.bar(win_state.window_x, win_state.window_y, win_state.window_w, win_state.window_h, WHITE);
}

pub fn line_input(ticker: &Ticker, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState, x: usize, y: usize, esc_ok: bool, max_chars: usize, max_width: usize, initial_input: &str) -> (String, bool) {
    let mut done = false;
    let mut result = false;
    let mut redraw = true;
    let mut update_cursor = false;
    let mut cursor_moved = true;
    let mut last_time = ticker.get_count();
    let mut cursor = new_cursor(initial_input.len());
    let mut input_str = String::from(initial_input);
    let mut old_str = String::from(initial_input);
    while !done {
        if update_cursor {
            cursor.xor_i(rdr, win_state, x, y, &input_str);
            update_cursor = false;
        }

        let mut c = input.last_ascii();
        let last_scan = input.last_scan();
        input.clear_last_scan();
        input.clear_last_ascii();

        let last_cursor_pos = cursor.pos;

        match last_scan {
            NumCode::LeftArrow => {
                cursor.pos = cursor.pos.saturating_sub(1);
                c = '\0';
                cursor_moved = true;
            },
            NumCode::RightArrow => {
                if cursor.pos < input_str.len() {
                    cursor.pos += 1;
                }
                c = '\0';
                cursor_moved = true;
            },
            NumCode::Home => {
                cursor.pos = 0;
                c = '\0';
                cursor_moved = true;
            },
            NumCode::End => {
                cursor.pos = input_str.len();
                c = '\0';
                cursor_moved = true;
            },
            NumCode::Return => {
                done = true;
                result = true;
                c = '\0';
            },
            NumCode::Escape => {
                if esc_ok {
                    done = true;
                    result = false;
                }
                c = '\0';
            },
            NumCode::BackSpace => {
                if cursor.pos > 0 {
                    input_str.remove(cursor.pos-1);
                    cursor.pos -= 1;
                    redraw = true;
                }
                c = '\0';
                cursor_moved = true;
            },
            NumCode::Delete => {
                if cursor.pos < input_str.len() {
                    input_str.remove(cursor.pos);
                    redraw = true;
                }
                c = '\0';
                cursor_moved = true;
            },
            NumCode::UpArrow|NumCode::DownArrow|NumCode::PgUp|NumCode::PgDn|NumCode::Insert => {
                c = '\0'
            },
            _ => {},
        }

        if c != '\0' {
            input_str.insert(cursor.pos, c);
            cursor.pos += 1;
            redraw = true;
        }

        if cursor_moved {
            update_cursor = false;
            cursor_moved = false;
            // clear the old cursor
            cursor.clear(rdr, win_state, x, y, &old_str, last_cursor_pos);
        }

        if redraw {
            let font = &rdr.fonts[win_state.font_number];
            // clear out old string and cursor
            cursor.clear(rdr, win_state, x, y, &old_str, last_cursor_pos);
            draw_string(rdr, font, &old_str, x, y, win_state.back_color);
            old_str = input_str.clone();
            draw_string(rdr, font, &input_str, x, y, win_state.font_color);
            redraw = false;
        }

        let count = ticker.get_count();
        if count - last_time > TICK_BASE / 2 {
            last_time = count;
            update_cursor = true;
        }
        if update_cursor {
            cursor.xor_i(rdr, win_state, x, y, &input_str); 
            update_cursor = false;
        }

        // don't poll to fast, otherwise key inputs will be missed
        while ticker.get_count() < (count + TICK_BASE/8) {}
    }

    if !result {
        let font = &rdr.fonts[win_state.font_number];
        draw_string(rdr, font, &old_str, x, y, win_state.back_color);
    }

    input.clear_keys_down();

    return (input_str, !result)
}

struct Cursor {
    status: bool,
    pos: usize,
}

fn new_cursor(pos: usize) -> Cursor {
    Cursor{status: false, pos}
}

impl Cursor {
    fn xor_i(&mut self, rdr: &VGARenderer, win_state: &mut WindowState, x: usize, y: usize, str: &str) {
        let font = &rdr.fonts[win_state.font_number];

        let str_before_cursor = &str[..self.pos];
        let (w, _) = measure_string(font, str_before_cursor);
        let px = x + w - 1;

        self.status ^= true;
        if self.status {
            draw_string(rdr, font, "\u{80}", px, y, win_state.font_color);
        } else {
            draw_string(rdr, font, "\u{80}", px, y, win_state.back_color);
        }
    }

    fn clear(&self, rdr: &VGARenderer, win_state: &mut WindowState, x: usize, y: usize, str: &str, pos: usize) {
        let font = &rdr.fonts[win_state.font_number];

        let str_before_cursor = &str[..pos];
        let (w, _) = measure_string(font, str_before_cursor);
        let px = x + w - 1;
        draw_string(rdr, font, "\u{80}", px, y, win_state.back_color);
    }
}