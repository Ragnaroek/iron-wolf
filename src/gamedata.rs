use std::io::{Read, Seek, SeekFrom};
use serde::{Serialize, Deserialize};

use crate::util;

#[derive(Serialize, Deserialize, Debug)]
pub struct GamedataHeader {
    pub offset: u32,
    pub length: u16
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GamedataHeaders {
    pub num_chunks : u16,
    pub sprite_start: u16,
    pub sound_start: u16,
    pub headers: Vec<GamedataHeader>,
}

#[derive(Serialize, Deserialize)]
pub struct TextureData {
    pub bytes: Vec<u8>
}

#[derive(Debug)]
pub struct SpritePost {
    pub start: usize,
    pub end: usize,
    pub pixel_offset: usize,
}

#[derive(Debug)]
pub struct SpriteData {
    pub left_pix: usize,
    pub right_pix: usize,
    pub pixel_pool: Vec<u8>,
    pub posts: Vec<Vec<SpritePost>>, // Vec of posts per column
}

fn empty_sprite_data() -> SpriteData {
    SpriteData{
        left_pix: 0,
        right_pix: 0,
        pixel_pool: Vec::new(),
        posts: Vec::new(),
    }
}

pub fn load_gamedata_headers(data: &Vec<u8>) -> Result<GamedataHeaders, String> {
    let mut reader = util::new_data_reader(&data);
    let num_chunks = reader.read_u16();
    let sprite_start = reader.read_u16();
    let sound_start = reader.read_u16();

    let mut headers = Vec::with_capacity(num_chunks as usize);
    for _ in 0..num_chunks {
        let offset = reader.read_u32();
        headers.push(GamedataHeader{offset, length: 0});
    }

    for i in 0..num_chunks as usize {
        let length = reader.read_u16();
        headers[i].length = length;
    }

    Ok(GamedataHeaders{num_chunks, sprite_start, sound_start, headers})
}

pub fn load_texture<M: Read+Seek>(data: &mut M, header: &GamedataHeader) -> Result<TextureData, String>{
        data.seek(SeekFrom::Start(header.offset as u64)).expect("seek failed");
        assert!(header.length == 4096 || header.length == 0); // textures should always be 64 x 64 pixels (or 0 for demo data)
        let mut buffer : Vec<u8> = vec![0; header.length as usize];
        let n = data.read(&mut buffer).expect("reading texture data failed");
        if n != header.length as usize {
            return Err("not enough bytes for texture in file".to_string());
        }
        Ok(TextureData{bytes: buffer})
}

pub fn load_all_textures<M: Read+Seek>(data: &mut M, headers: &GamedataHeaders) -> Result<Vec<TextureData>, String> {
    let mut result = Vec::with_capacity(headers.sprite_start as usize);
    
    for i in 0..(headers.sprite_start-1) {
        let texture = load_texture(data, &headers.headers[i as usize])?;
        result.push(texture);
    }

    Ok(result)
}

pub fn load_sprite<M: Read+Seek>(data: &mut M, header: &GamedataHeader) -> Result<SpriteData, String>{
    if header.offset == 0 || header.length == 0 {
        return Ok(empty_sprite_data());
    }
    
    data.seek(SeekFrom::Start(header.offset as u64)).expect("seek failed");
    let mut buffer : Vec<u8> = vec![0; header.length as usize];
    let n = data.read(&mut buffer).expect("reading sprite data failed");
    if n != header.length as usize {
        return Err("not enough bytes for sprite in file".to_string());
    }

    let mut reader = util::new_data_reader(&buffer);
    let left_pix = reader.read_u16() as usize;
    let right_pix = reader.read_u16() as usize;
    
    let len = (right_pix - left_pix)+1;
    let mut data_ofs = Vec::with_capacity(len);
    for _ in 0..len {
        data_ofs.push(reader.read_u16() as usize);
    }

    let pixel_buf_len = data_ofs[0] - (data_ofs.len()*2+4);
    let mut pixel_pool: Vec<u8> = Vec::with_capacity(pixel_buf_len);
    for _ in 0..pixel_buf_len {
        pixel_pool.push(reader.read_u8());
    }

    let mut pb_offset = 0;
    let mut posts = Vec::with_capacity(len);
    for mut post_start in data_ofs {
        let mut column = Vec::new();
        loop {
            let end = u16::from_le_bytes(buffer[post_start..post_start+2].try_into().unwrap())/2;
            if end == 0 {
                break;
            } 
            //[post_start+2..post_start+4] is a magical pixel buffer offset, but haven't figured out how this works. So computing
            // the offset here linearly from the left edge of the sprite
            let start = u16::from_le_bytes(buffer[post_start+4..post_start+6].try_into().unwrap())/2;
            column.push(SpritePost{
                start: start as usize,
                end: end as usize,
                pixel_offset: pb_offset
            });
            pb_offset += (end - start) as usize;
            post_start += 6;
        }
        posts.push(column);
    }

    return Ok(SpriteData{
        left_pix,
        right_pix,
        posts,
        pixel_pool,
    });
}

pub fn load_all_sprites<M: Read+Seek>(data: &mut M, headers: &GamedataHeaders) -> Result<Vec<SpriteData>, String> {
    let mut result = Vec::with_capacity(headers.sound_start as usize - headers.sprite_start as usize);
    
    for i in headers.sprite_start..(headers.sound_start-1) {
        let sprite = load_sprite(data, &headers.headers[i as usize])?;
        result.push(sprite);
    }

    Ok(result)
}