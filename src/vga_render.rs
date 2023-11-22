use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use vga::{CRTReg, SCReg, VGA};
use vga::util;

use super::assets::{Graphic, GraphicNum, GAMEPAL};
use super::vl;

pub const SCREENBWIDE: usize = 80;
pub const SCREEN_SIZE: usize = SCREENBWIDE * 208;

pub const PAGE_1_START: usize = 0;
pub const PAGE_2_START: usize = SCREEN_SIZE;
pub const PAGE_3_START: usize = SCREEN_SIZE*2;

static PIXMASKS: [u8; 4] = [1, 2, 4, 8];
static LEFTMASKS: [u8; 4]	= [15, 14, 12, 8];
static RIGHTMASKS: [u8; 4] = [1, 3, 7, 15];

pub struct VGARenderer {
	vga: Arc<VGA>,
	linewidth: usize,
	bufferofs: AtomicUsize,
	graphics: Vec<Graphic>,
}

pub fn init(vga: Arc<VGA>, graphics: Vec<Graphic>) -> VGARenderer {
	VGARenderer {
		vga,
		linewidth: 80,
		bufferofs: AtomicUsize::new(PAGE_1_START),
		graphics,
	}
}

impl VGARenderer {
	pub fn mem_to_screen(&self, data: &Vec<u8>, width: usize, height: usize, x: usize, y: usize) {
		let width_bytes = width >> 2;
		let mut mask = 1 << (x & 3);
		let mut src_ix = 0;
		let dst_offset = self.y_offset(y)+(x >> 2); 
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

	pub fn y_offset(&self, y: usize) -> usize {
		self.bufferofs.load(std::sync::atomic::Ordering::Relaxed) + y * self.linewidth
	}

	pub fn set_buffer_offset(&self, offset: usize) {
		self.bufferofs.store(offset, std::sync::atomic::Ordering::Relaxed);
	}

	pub fn buffer_offset(&self) -> usize {
		self.bufferofs.load(std::sync::atomic::Ordering::Relaxed)
	}

    pub async fn activate_buffer(&self, offset: usize) {
        util::display_enable(&self.vga).await;

        let addr_parts = offset.to_le_bytes();
        self.vga.set_crt_data(CRTReg::StartAdressLow, addr_parts[0]);
        self.vga.set_crt_data(CRTReg::StartAdressHigh, addr_parts[1]);

        util::vsync(&self.vga).await;
    }

    pub fn write_mem(&self, offset: usize, v_in: u8) {
        self.vga.write_mem(offset, v_in)
    }

	pub fn set_mask(&self, m: u8) {
		self.vga.set_sc_data(SCReg::MapMask, m)
	}

	pub fn bar(&self, x: usize, y: usize, width: usize, height: usize, color: u8) {
		let leftmask = LEFTMASKS[x & 3];
		let rightmask = RIGHTMASKS[(x + width - 1) & 3];
		let midbytes = ((x as i32 + (width as i32) + 3) >> 2) - (x as i32 >> 2) - 2;

		let mut dest = self.y_offset(y) + (x >> 2);

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
				self.vga.set_sc_data(SCReg::MapMask, rightmask);
				self.vga.write_mem(dest, color);

				dest += linedelta;
			}
		}

		self.vga.set_sc_data(SCReg::MapMask, 0xFF);
	}

	pub fn hlin(&self, x: usize, y: usize, width: usize, color: u8) {
		let xbyte = x >> 2;
		let leftmask = LEFTMASKS[x&3];
		let rightmask = RIGHTMASKS[(x+width-1)&3];
		let midbytes : i32 = ((x+width+3)>>2) as i32 - xbyte as i32 - 2;

		let mut dest = self.y_offset(y) + xbyte;
		if midbytes < 0 {
			self.vga.set_sc_data(SCReg::MapMask, leftmask & rightmask);
			self.vga.write_mem(dest, color);	
		} else {
			self.vga.set_sc_data(SCReg::MapMask, leftmask);
			self.vga.write_mem(dest, color);
			dest += 1;

			self.vga.set_sc_data(SCReg::MapMask, 0xFF);
			for _ in 0..midbytes {
				self.vga.write_mem(dest, color);
				dest += 1;
			}

			self.vga.set_sc_data(SCReg::MapMask, rightmask);
			self.vga.write_mem(dest, color);
		}

		self.vga.set_sc_data(SCReg::MapMask, 0xFF);
	}

	pub fn vlin(&self, x: usize, y: usize, height: usize, color: u8) {
		let mask = PIXMASKS[x&3];
		self.vga.set_sc_data(SCReg::MapMask, mask);

		let mut dest = self.y_offset(y) + (x >> 2);
		let mut h = height;
		while h > 0 {
			self.vga.write_mem(dest, color);
			dest += self.linewidth;
			h -= 1;
		}

		self.vga.set_sc_data(SCReg::MapMask, 0xFF);
	}

	pub fn plot(&self, x: usize, y: usize, color: u8) {
		let mask = PIXMASKS[x&3];
		self.vga.set_sc_data(SCReg::MapMask, mask);
		let dest = self.y_offset(y) + (x >> 2);
		self.vga.write_mem(dest, color);	
	}

	pub fn pic(&self, x: usize, y: usize, picnum: GraphicNum) {
		let graphic = &self.graphics[picnum as usize];
		self.mem_to_screen(&graphic.data, graphic.width, graphic.height, x & !7, y);
	}

    pub fn debug_pic(&self, data: &Vec<u8>) {
        for tex_y in 0..64 {
            for tex_x in 0..64 {
                self.plot(tex_x, tex_y, data[tex_x * 64+tex_y]);
            }
        }
    }

	pub async fn fade_out(&self) {
		vl::fade_out(&self.vga, 0, 255, 0, 0, 0, 30).await;
	}

	pub async fn fade_in(&self) {
		vl::fade_in(&self.vga,0, 255, GAMEPAL, 30).await;
	}
}