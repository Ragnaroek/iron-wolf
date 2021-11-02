use std::sync::Arc;

use vgaemu::{set_vertical_display_end, SCReg};

const MAXSCANLINES: usize = 200;
const PAGE_0_START: usize = 0;
const PAGE_1_START: usize = MAXSCANLINES;

pub trait Renderer {
	fn bar(&self, x: usize, y: usize, width: usize, height: usize, color: u8);
}

pub struct VGARenderer {
	vga: Arc<vgaemu::VGA>,
	linewidth: usize,
	bufferofs: usize,
}

pub fn init(vga: Arc<vgaemu::VGA>) -> VGARenderer {
	VGARenderer {
		vga,
		linewidth: 80,
		bufferofs: PAGE_0_START,
	}
}

static LEFTMASKS: [u8; 4] = [15, 14, 12, 8];
static RIGHTMASKS: [u8; 4] = [1, 3, 7, 15];

impl Renderer for VGARenderer {
	fn bar(&self, x: usize, y: usize, width: usize, height: usize, color: u8) {
		let leftmask = LEFTMASKS[x & 3];
		let rightmask = RIGHTMASKS[(x + width - 1) & 3];
		let midbytes = ((x as i32 + (width as i32) + 3) >> 2) - (x as i32 >> 2) - 2;

		let mut dest = self.bufferofs + y * self.linewidth;

		if midbytes < 0 {
			self.vga.set_sc_data(SCReg::MapMask, leftmask & rightmask);
			for _ in 0..height {
				self.vga.write_mem(dest, color);
				dest += self.linewidth;
			}
		} else {
			for _ in 0..height {
				let linedelta = self.linewidth - (midbytes as usize + 1);
				self.vga.set_sc_data(SCReg::MapMask, leftmask);
				self.vga.write_mem(dest, color);
				dest += 1;

				self.vga.set_sc_data(SCReg::MapMask, 0xFF);
				for _ in 0..midbytes {
					self.vga.write_mem(dest, color);
					dest += 1;
				}
				self.vga.set_sc_data(SCReg::MapMask, leftmask);
				self.vga.write_mem(dest, color);

				dest += linedelta;
			}
		}

		self.vga.set_sc_data(SCReg::MapMask, 0xFF);
	}
}
