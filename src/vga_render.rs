use std::sync::Arc;

use vgaemu::{SCReg};

use super::assets::{Graphic, GraphicNum, GAMEPAL};
use super::vl;

const MAXSCANLINES: usize = 200;
const PAGE_0_START: usize = 0;
const PAGE_1_START: usize = MAXSCANLINES;

pub trait Renderer {
	fn bar(&self, x: usize, y: usize, width: usize, height: usize, color: u8);
	fn pic(&self, x: usize, y: usize, picnum: GraphicNum);
	fn fade_out(&self);
	fn fade_in(&self);
}

pub struct VGARenderer {
	vga: Arc<vgaemu::VGA>,
	linewidth: usize,
	bufferofs: usize,
	graphics: Vec<Graphic>,
}

pub fn init(vga: Arc<vgaemu::VGA>, graphics: Vec<Graphic>) -> VGARenderer {
	VGARenderer {
		vga,
		linewidth: 80,
		bufferofs: PAGE_0_START,
		graphics,
	}
}

static LEFTMASKS: [u8; 4] = [15, 14, 12, 8];
static RIGHTMASKS: [u8; 4] = [1, 3, 7, 15];

impl VGARenderer {
	fn mem_to_screen(&self, data: &Vec<u8>, width: usize, height: usize, x: usize, y: usize) {
		let width_bytes = width >> 2;
		let mut mask = 1 << (x & 3);
		let mut src_ix = 0;
		let dst_offset = self.bufferofs + self.ylookup(y)+(x >> 2); 
		let mut dst_ix;
		for _ in 0..4 {
			self.vga.set_sc_data(SCReg::MapMask, mask);
			mask <<= 1;
			if mask == 16 {
				mask = 1;
			}
			dst_ix = dst_offset;
			for _ in 0..height {
				self.vga.write_mem_chunk(dst_ix, &data[src_ix..(src_ix+width_bytes)]);
				dst_ix += self.linewidth;
				src_ix += width_bytes;
			}
		}
	}

	fn ylookup(&self, y: usize) -> usize {
		y * self.linewidth
	}
}

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

	fn pic(&self, x: usize, y: usize, picnum: GraphicNum) {
		let graphic = &self.graphics[picnum as usize];
		println!("sizes = {}, {}", graphic.width, graphic.height);
		self.mem_to_screen(&graphic.data, graphic.width, graphic.height, x & !7, y);
	}

	fn fade_out(&self) {
		vl::fade_out(&self.vga, 0, 255, 0, 0, 0, 30)
	}

	fn fade_in(&self) {
		vl::fade_in(&self.vga,0, 255, GAMEPAL, 30);
	}
}




