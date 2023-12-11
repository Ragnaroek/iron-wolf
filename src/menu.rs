use crate::vga_render::VGARenderer;

const STRIPE : u8 = 0x2c;
const BORDERCOLOR : u8 = 0x29;

pub fn draw_stripes(rdr: &VGARenderer, y: usize) {
    rdr.bar(0, y, 320, 24, 0);
    rdr.hlin(0, 319, y+22, STRIPE);
}

pub fn clear_ms_screen(rdr: &VGARenderer) {
    rdr.bar(0, 0, 320, 200, BORDERCOLOR)
}