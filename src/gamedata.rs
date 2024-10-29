use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::str;

use opl::{AdlSound, Instrument};

use crate::assets::{SoundName, DIGI_MAP};
use crate::def::DigiSound;
use crate::sd::{DigiInfo, Sound};
use crate::{assets::WolfVariant, util};

#[derive(Serialize, Deserialize, Debug)]
pub struct GamedataHeader {
    pub offset: u32,
    pub length: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GamedataHeaders {
    pub num_chunks: u16,
    pub sprite_start: u16,
    pub sound_start: u16,
    pub headers: Vec<GamedataHeader>,
}

#[derive(Serialize, Deserialize)]
pub struct TextureData {
    pub bytes: Vec<u8>,
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
    SpriteData {
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
        headers.push(GamedataHeader { offset, length: 0 });
    }

    for i in 0..num_chunks as usize {
        let length = reader.read_u16();
        headers[i].length = length;
    }

    Ok(GamedataHeaders {
        num_chunks,
        sprite_start,
        sound_start,
        headers,
    })
}

pub fn load_texture<M: Read + Seek>(
    data: &mut M,
    header: &GamedataHeader,
) -> Result<TextureData, String> {
    data.seek(SeekFrom::Start(header.offset as u64))
        .expect("seek failed");
    assert!(
        header.length == 4096 || header.length == 0,
        "header length was {}, but should be 4096 or 0 for a texture",
        header.length
    ); // textures should always be 64 x 64 pixels (or 0 for demo data)
    let mut buffer: Vec<u8> = vec![0; header.length as usize];
    let n = data.read(&mut buffer).expect("reading texture data failed");
    if n != header.length as usize {
        return Err("not enough bytes for texture in file".to_string());
    }
    Ok(TextureData { bytes: buffer })
}

pub fn load_all_textures<M: Read + Seek>(
    data: &mut M,
    headers: &GamedataHeaders,
) -> Result<Vec<TextureData>, String> {
    let mut result = Vec::with_capacity(headers.sprite_start as usize);

    for i in 0..(headers.sprite_start - 1) {
        let texture = load_texture(data, &headers.headers[i as usize])?;
        result.push(texture);
    }

    Ok(result)
}

pub fn load_sprite<M: Read + Seek>(
    data: &mut M,
    header: &GamedataHeader,
) -> Result<SpriteData, String> {
    if header.offset == 0 || header.length == 0 {
        return Ok(empty_sprite_data());
    }

    data.seek(SeekFrom::Start(header.offset as u64))
        .expect("seek failed");
    let mut buffer: Vec<u8> = vec![0; header.length as usize];
    let n = data.read(&mut buffer).expect("reading sprite data failed");
    if n != header.length as usize {
        return Err("not enough bytes for sprite in file".to_string());
    }

    let mut reader = util::new_data_reader(&buffer);
    let left_pix = reader.read_u16() as usize;
    let right_pix = reader.read_u16() as usize;

    let len = (right_pix - left_pix) + 1;
    let mut data_ofs = Vec::with_capacity(len);
    for _ in 0..len {
        data_ofs.push(reader.read_u16() as usize);
    }

    let pixel_buf_len = data_ofs[0] - (data_ofs.len() * 2 + 4);
    let mut pixel_pool: Vec<u8> = Vec::with_capacity(pixel_buf_len);
    for _ in 0..pixel_buf_len {
        pixel_pool.push(reader.read_u8());
    }

    let mut pb_offset = 0;
    let mut posts = Vec::with_capacity(len);
    for mut post_start in data_ofs {
        let mut column = Vec::new();
        loop {
            let end =
                u16::from_le_bytes(buffer[post_start..post_start + 2].try_into().unwrap()) / 2;
            if end == 0 {
                break;
            }
            //[post_start+2..post_start+4] is a magical pixel buffer offset, but haven't figured out how this works. So computing
            // the offset here linearly from the left edge of the sprite
            let start =
                u16::from_le_bytes(buffer[post_start + 4..post_start + 6].try_into().unwrap()) / 2;
            column.push(SpritePost {
                start: start as usize,
                end: end as usize,
                pixel_offset: pb_offset,
            });
            pb_offset += (end - start) as usize;
            post_start += 6;
        }
        posts.push(column);
    }

    return Ok(SpriteData {
        left_pix,
        right_pix,
        posts,
        pixel_pool,
    });
}

pub fn load_all_sprites<M: Read + Seek>(
    data: &mut M,
    headers: &GamedataHeaders,
) -> Result<Vec<SpriteData>, String> {
    let mut result =
        Vec::with_capacity(headers.sound_start as usize - headers.sprite_start as usize);

    for i in headers.sprite_start..(headers.sound_start - 1) {
        let sprite = load_sprite(data, &headers.headers[i as usize])?;
        result.push(sprite);
    }

    Ok(result)
}

pub fn load_all_digi_sounds<M: Read + Seek>(
    sound: &Sound,
    data: &mut M,
    headers: &GamedataHeaders,
) -> Result<HashMap<SoundName, DigiSound>, String> {
    let sound_info_page = load_page(data, headers, (headers.num_chunks - 1) as usize)?;
    let num_digi = (headers.headers[(headers.num_chunks - 1) as usize].length / 4) as usize;

    let mut digi_list = Vec::with_capacity(num_digi as usize);
    for i in 0..num_digi {
        let start_page =
            u16::from_le_bytes(sound_info_page[(i * 4)..(i * 4 + 2)].try_into().unwrap()) as usize;
        let length = u16::from_le_bytes(
            sound_info_page[(i * 4 + 2)..(i * 4 + 4)]
                .try_into()
                .unwrap(),
        ) as usize;
        if start_page >= headers.num_chunks as usize {
            break;
        }
        digi_list.push(DigiInfo { start_page, length })
    }

    let mut sounds = HashMap::new();
    for digi_sound in &DIGI_MAP {
        let digi = &digi_list[digi_sound.page_no];
        let digi_data = load_digi_page(data, headers, digi)?;
        sounds.insert(
            digi_sound.sound,
            sound.prepare_digi_sound(digi_sound.channel, digi_data)?,
        );
    }
    Ok(sounds)
}

fn load_digi_page<M: Read + Seek>(
    data: &mut M,
    headers: &GamedataHeaders,
    digi: &DigiInfo,
) -> Result<Vec<u8>, String> {
    let header = &headers.headers[headers.sound_start as usize + digi.start_page];
    data.seek(SeekFrom::Start(header.offset as u64))
        .map_err(|e| e.to_string())?;
    let mut buffer: Vec<u8> = vec![0; digi.length as usize];
    let n = data.read(&mut buffer).map_err(|e| e.to_string())?;
    if n != digi.length as usize {
        return Err("not enough bytes in page".to_string());
    }
    Ok(buffer)
}

fn load_page<M: Read + Seek>(
    data: &mut M,
    headers: &GamedataHeaders,
    page: usize,
) -> Result<Vec<u8>, String> {
    let header = &headers.headers[page];
    data.seek(SeekFrom::Start(header.offset as u64))
        .map_err(|e| e.to_string())?;
    let mut buffer: Vec<u8> = vec![0; header.length as usize];
    let n = data.read(&mut buffer).map_err(|e| e.to_string())?;
    if n != header.length as usize {
        return Err("not enough bytes in page".to_string());
    }
    Ok(buffer)
}

pub fn load_audio_headers<M: Read>(data: &mut M) -> Result<Vec<u32>, String> {
    let mut buf = Vec::new();
    let size = data.read_to_end(&mut buf).map_err(|e| e.to_string())?;

    let num_headers = size / 4;
    let mut headers = Vec::with_capacity(num_headers);
    for i in 0..num_headers {
        let offset = u32::from_le_bytes(buf[(i * 4)..((i * 4) + 4)].try_into().unwrap());
        headers.push(offset)
    }

    Ok(headers)
}

pub fn load_audio_sounds<M: Read + Seek>(
    headers: &Vec<u32>,
    data: &mut M,
    variant: &WolfVariant,
) -> Result<Vec<AdlSound>, String> {
    let mut sounds = Vec::with_capacity(variant.start_digi_sound - variant.start_adlib_sound);
    for chunk_no in variant.start_adlib_sound..variant.start_digi_sound {
        let offset = headers[chunk_no];
        let size = (headers[chunk_no + 1] - offset) as usize;
        let mut data_buf = vec![0; size];
        data.seek(SeekFrom::Start(offset as u64))
            .map_err(|e| e.to_string())?;
        data.read_exact(&mut data_buf).map_err(|e| e.to_string())?;
        sounds.push(read_sound(data_buf));
    }
    Ok(sounds)
}

fn read_sound(data: Vec<u8>) -> AdlSound {
    let length = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let instrument = Instrument {
        m_char: data[6],
        c_char: data[7],
        m_scale: data[8],
        c_scale: data[9],
        m_attack: data[10],
        c_attack: data[11],
        m_sus: data[12],
        c_sus: data[13],
        m_wave: data[14],
        c_wave: data[15],
        n_conn: data[16],
        voice: data[17],
        mode: data[18],
        // data[19..22] are padding and omitted
    };
    AdlSound {
        length,
        priority: u16::from_le_bytes(data[4..6].try_into().unwrap()),
        instrument,
        block: data[22],
        data: data[23..(23 + length as usize)].to_vec(),
        terminator: data[23 + length as usize],
        name: str::from_utf8(&data[(23 + length as usize) + 1..data.len() - 1])
            .expect("sound name")
            .to_string(),
    }
}
