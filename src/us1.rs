use crate::{assets::Font, def::UserState, vga_render::VGARenderer};

pub fn print(rdr: &VGARenderer, font: &Font, user_state: &mut UserState, str: &str, px_in: usize, py_in: usize, color: u8) {
    let lines = str.split("\n");
    
    let mut px = px_in;
    let mut py = py_in;
    for line in lines {
        let (_, h) = measure_string(font, line);
        draw_string(rdr, font, line, px, py, color);
        px = user_state.window_x;
        py += h;
    }
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