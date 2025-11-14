use vga::VGA;

use crate::{
    assets::GAMEPAL,
    rc::RenderContext,
    vl::{fade_in, fade_out},
};

pub const WHITE: u8 = 15;
pub const BLACK: u8 = 0;

pub fn vw_hlin(rc: &mut RenderContext, x: usize, z: usize, y: usize, color: u8) {
    rc.hlin(x, y, z - x + 1, color);
}

pub fn vw_vlin(rc: &mut RenderContext, y: usize, z: usize, x: usize, color: u8) {
    rc.vlin(x, y, z - y + 1, color)
}

pub async fn vw_fade_out(vga: &mut VGA) {
    fade_out(vga, 0, 255, 0, 0, 0, 30).await
}

pub async fn vw_fade_in(vga: &mut VGA) {
    fade_in(vga, 0, 255, &GAMEPAL, 30).await
}

pub fn draw_tile_8(rc: &mut RenderContext, x: usize, y: usize, tile: usize) {
    rc.tile_to_screen(tile, 8, 8, x, y)
}
