use crate::vga_render::VGARenderer;

pub fn vw_hlin(rdr: &VGARenderer, x: usize, z: usize, y: usize, color: u8) {
    rdr.hlin(x, y, z-x+1, color);
}

pub fn vw_vlin(rdr: &VGARenderer, y: usize, z: usize, x: usize, color: u8) {
    rdr.vlin(x, y, z-y+1, color)
}