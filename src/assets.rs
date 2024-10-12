use super::util;

use super::config::IWConfig;

pub static GAMEPAL: &'static [u8] = include_bytes!("../assets/gamepal.bin");

pub static GRAPHIC_DICT: &'static str = "VGADICT.WL6";
pub static GRAPHIC_HEAD: &'static str = "VGAHEAD.WL6";
pub static GRAPHIC_DATA: &'static str = "VGAGRAPH.WL6";

pub enum GraphicNum {
	STATUSBARPIC = 86,
	TITLEPIC = 87,
	PG13PIC = 88,
	CREDITSPIC = 89,
}

const STRUCTPIC: usize = 0;
const STARTPICS: usize = 3;
const STARTTILE8: usize = 150;
const STARTEXTERNS: usize = 136;
const NUM_PICS: usize = 132;

pub struct Graphic {
	pub data: Vec<u8>,
	pub width: usize,
	pub height: usize,
}

pub struct Huffnode {
	bit0: u16,
	bit1: u16,
}

pub fn load_all_graphics(config: &IWConfig) -> Result<Vec<Graphic>, String> {
	let grhuffman_bytes = util::load_file(&config.wolf3d_data.join(GRAPHIC_DICT));
	let grhuffman = to_huffnodes(grhuffman_bytes);

	let grstarts = util::load_file(&config.wolf3d_data.join(GRAPHIC_HEAD));
	let grdata = util::load_file(&config.wolf3d_data.join(GRAPHIC_DATA));

	let picsizes = extract_picsizes(&grdata, &grstarts, &grhuffman);

	let mut graphics = Vec::with_capacity(NUM_PICS);
	for _ in 0..10 {
		graphics.push(Graphic{data: Vec::with_capacity(0), width: 0, height: 0});
	}
	for i in 10..NUM_PICS {
		let g = load_graphic(
			i,
			&grstarts,
			&grdata,
			&grhuffman,
			&picsizes
		)?;
		graphics.push(g);
	}

	Ok(graphics)
}

fn extract_picsizes(grdata: &Vec<u8>, grstarts: &Vec<u8>, grhuffman: &Vec<Huffnode>) -> Vec<(usize, usize)> {
	
	let (complen, explen) = gr_chunk_length(STRUCTPIC, grdata, grstarts);
	let f_offset = (grfilepos(STRUCTPIC, grstarts) + 4) as usize;
	let expanded = huff_expand(&grdata[f_offset..(f_offset+complen)], explen, grhuffman);
	
	assert_eq!(explen/4, NUM_PICS); // otherwise the data file may not match the code

	let mut picsizes = Vec::with_capacity(NUM_PICS);
	let mut offset = 0;

	// TODO Write util functions for from_le_bytes()..try_into.unwrap noise
	for _ in 0..(explen/4) {
		let width = i16::from_le_bytes(expanded[offset..(offset+2)].try_into().unwrap()) as usize;
		let height = i16::from_le_bytes(expanded[offset + 2..(offset+4)].try_into().unwrap()) as usize;
		picsizes.push(
			(width as usize, height as usize)
		);
		offset += 4;
	}

	picsizes
}

fn gr_chunk_length(chunk: usize, grdata: &Vec<u8>, grstarts: &Vec<u8>) -> (usize, usize) {
	let file_offset = grfilepos(chunk, grstarts) as usize;
	let chunkexplen = u32::from_le_bytes(grdata[file_offset..(file_offset+4)].try_into().unwrap());
	(grfilepos(chunk+1, grstarts) as usize - file_offset - 4, chunkexplen as usize)
}

fn to_huffnodes(bytes: Vec<u8>) -> Vec<Huffnode> {
	let mut nodes = Vec::with_capacity(255);

	let mut offset = 0;
	for _ in 0..255 {
		let bit0 = u16::from_le_bytes(bytes[offset..(offset + 2)].try_into().unwrap());
		let bit1 = u16::from_le_bytes(bytes[(offset + 2)..(offset + 4)].try_into().unwrap());
		nodes.push(Huffnode {
			bit0: bit0,
			bit1: bit1,
		});
		offset += 4;
	}

	nodes
}

fn load_graphic(
	chunk: usize,
	grstarts: &Vec<u8>,
	grdata: &Vec<u8>,
	grhuffman: &Vec<Huffnode>,
	picsizes: &Vec<(usize, usize)>,
) -> Result<Graphic, String> {
	let pos_int = grfilepos(chunk, grstarts);
	if pos_int < 0 {
		return Err(format!("could not load chunk {}", pos_int));
	}
	let pos = pos_int as usize;
	let mut next = chunk + 1;
	while grfilepos(next, grstarts) == -1 {
		next += 1;
	}

	let compressed = (grfilepos(next, grstarts) - pos_int) as usize;
	let source = &grdata[pos..(pos + compressed)];

	Ok(expand_graphic(chunk, source, grhuffman, picsizes))
}

fn grfilepos(chunk: usize, grstarts: &Vec<u8>) -> i32 {
	let offset = chunk * 3;
	let mut value = i32::from_le_bytes(grstarts[offset..(offset + 4)].try_into().unwrap());
	value &= 0x00ffffff;
	if value == 0xffffff {
		-1
	} else {
		value
	}
}

fn expand_graphic(chunk: usize, data: &[u8], grhuffman: &Vec<Huffnode>, picsizes: &Vec<(usize, usize)>) -> Graphic {
	if chunk >= STARTTILE8 && chunk < STARTEXTERNS {
		panic!("TILE Expand not yet implemented");
	}

	let len = i32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
	let expanded = huff_expand(&data[4..], len, grhuffman);
	let size = picsizes[chunk-STARTPICS];
	return Graphic {
		data: expanded,
		width: size.0,
		height: size.1,
	};
}

fn huff_expand(data: &[u8], len: usize, grhuffman: &Vec<Huffnode>) -> Vec<u8> {
	let mut expanded = Vec::with_capacity(len);
	let head = &grhuffman[254];
	if len < 0xfff0 {
		let mut node = head;
		let mut read = 0;
		let mut input = data[read];
		read += 1;
		let mut mask: u8 = 0x01;
		while expanded.len() < len {
			let node_value = if (input & mask) == 0 {
				// bit not set
				node.bit0
			} else {
				node.bit1
			};

			if mask == 0x80 {
				input = data[read];
				read += 1;
				mask = 1;
			} else {
				mask <<= 1;
			}

			if node_value < 256 {
				// leaf node, dx is the uncompressed byte!
				expanded.push(node_value as u8);
				node = head;
			} else {
				// -256 here, since the huffman optimisation is not done
				node = &grhuffman[(node_value-256) as usize];
			}
		}
	} else {
		panic!("implement expand 64k data");
	}
	expanded
}