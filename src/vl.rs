use vgaemu::{ColorReg};

pub fn set_palette(vga: &vgaemu::VGA, palette: &[u8]) {
	debug_assert_eq!(palette.len(), 768);
    for i in 0..768 {
        vga.set_color_reg(ColorReg::Data, palette[i]);
    }
}