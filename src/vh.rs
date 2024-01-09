use vga::VGA;

use crate::{vga_render::VGARenderer, vl::{fade_out, fade_in}, assets::GAMEPAL};

pub const WHITE : u8 = 15;

pub fn vw_hlin(rdr: &VGARenderer, x: usize, z: usize, y: usize, color: u8) {
    rdr.hlin(x, y, z-x+1, color);
}

pub fn vw_vlin(rdr: &VGARenderer, y: usize, z: usize, x: usize, color: u8) {
    rdr.vlin(x, y, z-y+1, color)
}

pub async fn vw_fade_out(vga: &VGA) {
    fade_out(vga, 0, 255, 0, 0, 0, 30).await
}

pub async fn vw_fade_in(vga: &VGA) {
    fade_in(vga, 0, 255, &GAMEPAL, 30).await
}

pub fn draw_tile_8(rdr: &VGARenderer, x: usize, y: usize, tile: usize) {
    rdr.mem_to_screen(&rdr.tiles.tile8[tile], 8, 8, x, y)
}