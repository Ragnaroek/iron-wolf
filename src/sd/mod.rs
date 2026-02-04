#[cfg(feature = "sdl")]
pub mod sd_sdl;
#[cfg(feature = "sdl")]
pub use sd_sdl::{DigiSound, Sound, startup};

#[cfg(feature = "web")]
pub mod sd_web;
#[cfg(feature = "web")]
pub use sd_web::{DigiSound, Sound, startup};

#[cfg(feature = "test")]
pub mod sd_sdl;
#[cfg(feature = "test")]
pub mod sd_tst;
#[cfg(feature = "test")]
pub use sd_sdl::DigiSound;
#[cfg(feature = "test")]
pub use sd_tst::{Sound, startup, test_sound};

#[cfg(feature = "test")]
#[path = "./mod_test.rs"]
mod mod_test;

use crate::{
    assets::{Music, SoundName, WolfFile},
    def::{Assets, TILESHIFT},
    draw::RayCast,
    fixed::{Fixed, fixed_by_frac},
    loader::Loader,
};

use opl::OPL;

pub const SOURCE_SAMPLE_RATE: i32 = 7042;
const MAX_TRACKS: usize = 10;
const ATABLE_MAX: i32 = 15;

const RIGHT_TABLE: [[u8; ATABLE_MAX as usize * 2]; ATABLE_MAX as usize] = [
    [
        8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 0, 0, 0, 0, 0, 1, 3, 5, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 6, 4, 0, 0, 0, 0, 0, 2, 4, 6, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 6, 6, 4, 1, 0, 0, 0, 1, 2, 4, 6, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 6, 5, 4, 2, 1, 0, 1, 2, 3, 5, 7, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 6, 5, 4, 3, 2, 2, 3, 3, 5, 6, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 6, 6, 5, 4, 4, 4, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 6, 6, 5, 5, 5, 6, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 6, 6, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
];

const LEFT_TABLE: [[u8; ATABLE_MAX as usize * 2]; ATABLE_MAX as usize] = [
    [
        8, 8, 8, 8, 8, 8, 8, 8, 5, 3, 1, 0, 0, 0, 0, 0, 6, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 6, 4, 2, 0, 0, 0, 0, 0, 4, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 6, 4, 2, 1, 0, 0, 0, 1, 4, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 7, 5, 3, 2, 1, 0, 1, 2, 4, 5, 6, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 6, 5, 3, 3, 2, 2, 3, 4, 5, 6, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 5, 4, 4, 4, 4, 5, 6, 6, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 6, 5, 5, 5, 6, 6, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 7, 6, 6, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
    [
        8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    ],
];

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum SoundMode {
    Off,
    PC,
    AdLib,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum DigiMode {
    Off,
    SoundSource,
    SoundBlaster,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum MusicMode {
    Off,
    AdLib,
}

#[derive(Copy, Clone)]
pub struct Modes {
    pub sound: SoundMode,
    pub digi: DigiMode,
    pub music: MusicMode,
}

pub fn default_modes() -> Modes {
    Modes {
        sound: SoundMode::AdLib,
        digi: DigiMode::SoundBlaster,
        music: MusicMode::AdLib,
    }
}

pub struct DigiInfo {
    pub start_page: usize,
    pub length: usize,
}

pub fn clear_music(opl: &mut OPL) -> Result<(), String> {
    opl.stop_imf()?;
    opl.write_reg(0xBD, 0)?;
    for i in 0..MAX_TRACKS as u32 {
        opl.write_reg(0xB0 + i + 1, 0)?;
    }
    Ok(())
}

pub fn load_track(track: Music, assets: &Assets, loader: &Loader) -> Vec<u8> {
    let variant = loader.variant();
    let trackno = track as usize;
    let offset = assets.audio_headers[variant.start_music + trackno];
    let len = assets.audio_headers[variant.start_music + trackno + 1] - offset;

    //read the full chunk with size bytes at the beginning and tags at the end
    let track_chunk = loader
        .load_wolf_file_slice(WolfFile::AudioData, offset as u64, len as usize)
        .expect("load track data");

    let track_size = u16::from_le_bytes(track_chunk[0..2].try_into().unwrap()) as usize;

    let mut track_data = vec![0; track_size];
    track_data.copy_from_slice(&track_chunk[2..(track_size + 2)]);
    track_data
}

pub fn check_sound_prio(
    playing_sound: &Option<SoundName>,
    assets: &Assets,
    sound: SoundName,
) -> bool {
    if playing_sound.is_some() {
        let playing_prio =
            assets.audio_sounds[playing_sound.expect("playing sound") as usize].priority;
        let new_sound_prio = assets.audio_sounds[sound as usize].priority;
        if new_sound_prio < playing_prio {
            return false;
        }
    }
    true
}

pub fn sound_loc(rc: &RayCast, gx_param: Fixed, gy_param: Fixed) -> (u8, u8) {
    let view_x = Fixed::new_from_i32(rc.view_x);
    let view_y = Fixed::new_from_i32(rc.view_y);

    let gx = gx_param - view_x;
    let gy = gy_param - view_y;

    // calculate newx
    let xt = fixed_by_frac(gx, rc.view_cos);
    let yt = fixed_by_frac(gy, rc.view_sin);
    let mut x = (xt - yt).to_i32() >> TILESHIFT;

    // calculate newy
    let xt = fixed_by_frac(gx, rc.view_sin);
    let yt = fixed_by_frac(gy, rc.view_cos);
    let mut y = (yt + xt).to_i32() >> TILESHIFT;

    if y >= ATABLE_MAX {
        y = ATABLE_MAX - 1;
    } else if y <= -ATABLE_MAX {
        y = -ATABLE_MAX;
    }

    if x < 0 {
        x = -x;
    }
    if x >= ATABLE_MAX {
        x = ATABLE_MAX - 1;
    }

    (
        LEFT_TABLE[x as usize][(y + ATABLE_MAX) as usize],
        RIGHT_TABLE[x as usize][(y + ATABLE_MAX) as usize],
    )
}
