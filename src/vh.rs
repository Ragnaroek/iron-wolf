use crate::vga_render::{VGARenderer, FREE_START};

pub const WHITE : u8 = 15;

pub fn vw_hlin(rdr: &VGARenderer, x: usize, z: usize, y: usize, color: u8) {
    rdr.hlin(x, y, z-x+1, color);
}

pub fn vw_vlin(rdr: &VGARenderer, y: usize, z: usize, x: usize, color: u8) {
    rdr.vlin(x, y, z-y+1, color)
}

pub fn draw_tile_8(rdr: &VGARenderer, x: usize, y: usize, tile: usize) {
    rdr.mem_to_screen(&rdr.tiles.tile8[tile], 8, 8, x, y)
}