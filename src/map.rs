use std::io::{Seek, SeekFrom, Read};
use serde::{Serialize, Deserialize};

use super::util;

pub const MAP_PLANES: usize = 2;
pub const NUM_MAPS: usize = 60;

pub fn rlew_expand(source: &[u8], len: usize, rlew_tag: u16) -> Vec<u16> {
	
	let mut expanded = Vec::with_capacity(len);

	let trail = source.len() % 2 != 0;
	let loop_len = if trail { len - 1 } else { len };

	let mut reader = util::new_data_reader(source);
	loop {
		let value = reader.read_u16();
		if value != rlew_tag {
			expanded.push(value);
		} else {
			let count = reader.read_u16();
			let value = reader.read_u16();
			for _ in 0..count {
				expanded.push(value);
			}
		}

		if expanded.len() >= loop_len {
			break;
		}
	}
	
	if trail {
		expanded.push(source[source.len()-1] as u16);
	}

	while expanded.len() < len {
		expanded.push(0);
	}

	expanded
}

const NEARTAG : u8 = 0xa7;
const FARTAG : u8 = 0xa8;

pub fn carmack_expand(data: &[u8], len: usize) -> Vec<u8> {
	let mut expanded = Vec::with_capacity(len);

	let mut length = len / 2;
	let mut in_ptr = 0;

	while length != 0 {
		let word_count = data[in_ptr]; 
		let ch_high = data[in_ptr+1];
		in_ptr += 2;

		if ch_high == NEARTAG {
			let offset = data[in_ptr];
			in_ptr += 1;

			if word_count == 0 {
				expanded.push(offset);
				expanded.push(NEARTAG);
				length -= 1;
			} else {
				let mut copy_ptr = expanded.len() - (offset as usize * 2);
				length -= word_count as usize;
				for _ in 0..(word_count as usize * 2) {
					expanded.push(expanded[copy_ptr]);
					copy_ptr += 1;
				}
			}
		} else if ch_high == FARTAG {
			let offset_low = data[in_ptr];
			in_ptr += 1;

			if word_count == 0 {
				expanded.push(offset_low);
				expanded.push(FARTAG);
				length -= 1;
			} else {
				let offset_high = data[in_ptr];
				in_ptr += 1;

				let mut offset = offset_high as usize;
				offset <<= 8;
				offset |= offset_low as usize;

				let mut copy_ptr = offset * 2;
				length -= word_count as usize;
				for _ in 0..(word_count as usize * 2) {
					expanded.push(expanded[copy_ptr]);
					copy_ptr += 1;
				}
			}
		} else {
			// add word as is (destructured here as count and ch_high)
			expanded.push(word_count);
			expanded.push(ch_high);
			length -= 1;
		}
	}

	// handle trailing byte at the end if len is odd
	if expanded.len() != len {
		expanded.push(data[in_ptr]);
	}

	while expanded.len() < len {
		expanded.push(0);
	}

	expanded
}

// map stuff

#[derive(Serialize, Deserialize)]
pub struct MapData {
	pub segs: [Vec<u16>; MAP_PLANES]
}

#[derive(Serialize, Deserialize)]
pub struct MapType {
	pub plane_start: [i32; 3],
	pub plane_length: [u16; 3],
	pub width: u16,
	pub height: u16,
	pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MapFileType {
	pub rlew_tag: u16,
	pub header_offsets: Vec<i32>,
}

pub fn load_map<M: Seek + Read>(map_data: &mut M, map_headers: &Vec<MapType>, map_offsets: &MapFileType, mapnum: usize) -> Result<MapData, String>{
 
	let mut segs = [Vec::with_capacity(0), Vec::with_capacity(0)];

	for plane in 0..MAP_PLANES {
		let pos = map_headers[mapnum].plane_start[plane];
		let compressed = map_headers[mapnum].plane_length[plane];

		let mut buf = vec![0; compressed as usize];
		map_data.seek(SeekFrom::Start(pos as u64)).expect("map seek failed");
		map_data.read_exact(&mut buf).expect("map read failed");

		let mut reader = util::new_data_reader(&buf);
		let expanded_len = reader.read_u16();		

		let remaining_bytes = reader.unread_bytes();

		let carmack_expanded = carmack_expand(remaining_bytes, expanded_len as usize);
		let expanded = rlew_expand(&carmack_expanded[2..], 64*64, map_offsets.rlew_tag);

		segs[plane] = expanded;
	}

	Ok(MapData{segs}) 
}

pub fn load_map_headers(bytes: &Vec<u8>, offsets: MapFileType) -> Result<(MapFileType, Vec<MapType>), String> {
	let mut headers = Vec::with_capacity(NUM_MAPS);
	for i in 0..NUM_MAPS {
		let pos = offsets.header_offsets[i];
		if pos < 0 {
			// skip sparse maps
			continue;
		}

		let mut reader = util::new_data_reader_with_offset(&bytes, pos as usize);

		let mut plane_start = [0; 3];
		for j in 0..3 {
			plane_start[j] = reader.read_i32();
		}

		let mut plane_length = [0; 3];
		for j in 0..3 {
			plane_length[j] = reader.read_u16();
		}

		let width = reader.read_u16();
		let height = reader.read_u16();
		let mut name = reader.read_utf8_string(16);
		name.retain(|c| c != '\0');

		headers.push(MapType{plane_start, plane_length, width, height, name});
	}
	Ok((offsets, headers))
}

pub fn load_map_offsets(bytes: &Vec<u8>) -> Result<MapFileType, String> {	
	let mut reader = util::new_data_reader(&bytes);
	let rlew_tag = reader.read_u16();
	
	let mut header_offsets = Vec::with_capacity(100);
	for _ in 0..100 {
		header_offsets.push(reader.read_i32());
	}
	Ok(MapFileType {
		rlew_tag,
		header_offsets,
	})
}