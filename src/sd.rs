#[cfg(feature = "test")]
#[path = "./sd_test.rs"]
mod sd_test;

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use tokio::runtime::Runtime;

use crate::{
    assets::{DigiChannel, Music, SoundName, WolfFile},
    def::{Assets, DigiSound, ObjType, TILESHIFT},
    draw::RayCast,
    fixed::{Fixed, fixed_by_frac},
    loader::Loader,
    start::quit,
};

use opl::{OPL, OPLSettings};

#[cfg(any(feature = "sdl", feature = "test"))]
use sdl2::audio::{self, AudioCVT, AudioFormat};
#[cfg(any(feature = "sdl", feature = "test"))]
use sdl2::mixer::{self, Channel};

#[cfg(feature = "web")]
use web_sys::{AudioContext, AudioContextOptions};

const SOURCE_SAMPLE_RATE: i32 = 7042;
const TARGET_SAMPLE_RATE: f32 = 44100.0;
const PLAYBACK_RATE: f32 = SOURCE_SAMPLE_RATE as f32 / TARGET_SAMPLE_RATE;

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
struct Modes {
    sound: SoundMode,
    digi: DigiMode,
    music: MusicMode,
}

fn default_modes() -> Modes {
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

#[cfg(any(feature = "sdl", feature = "test"))]
pub struct DigiMixConfig {
    pub frequency: i32,
    pub format: AudioFormat,
    pub channels: i32,
    pub group: mixer::Group,
}

#[cfg(feature = "sdl")]
pub struct Sound {
    modes: Arc<Mutex<Modes>>,
    opl: Arc<Mutex<OPL>>,
    mix_config: Arc<Mutex<DigiMixConfig>>,
    rt: Arc<Runtime>,
    sound_playing: Arc<Mutex<Option<SoundName>>>,
    left_pos: u8,
    right_pos: u8,
}

#[cfg(feature = "test")]
pub struct Sound {}

#[cfg(feature = "web")]
pub struct Sound {
    modes: Modes,
    pub opl: OPL,
    sound_playing: Option<SoundName>,
    left_pos: u8,
    right_pos: u8,
    digi_context: AudioContext,
}

#[cfg(feature = "test")]
pub fn test_sound() -> Sound {
    Sound {}
}

#[cfg(feature = "test")]
pub fn startup(_rt: Arc<Runtime>) -> Result<Sound, String> {
    Ok(test_sound())
}

#[cfg(any(feature = "sdl", feature = "test"))]
const OPL_SETTINGS: OPLSettings = OPLSettings {
    mixer_rate: 49716,
    imf_clock_rate: 700,
    adl_clock_rate: 140,
};

#[cfg(feature = "web")]
const OPL_SETTINGS: OPLSettings = OPLSettings {
    imf_clock_rate: 700,
    adl_clock_rate: 140,
};

#[cfg(feature = "sdl")]
pub fn startup(rt: Arc<Runtime>) -> Result<Sound, String> {
    let mut opl = OPL::new()?;
    opl.init(OPL_SETTINGS);

    mixer::open_audio(44100, mixer::AUDIO_S16LSB, 2, 2048)?;
    let (mix_freq, mix_format, mix_channels) = mixer::query_spec()?;
    mixer::reserve_channels(2);
    let group = mixer::Group(1);
    group.add_channels_range(2, 8 - 1);

    let mix_config = DigiMixConfig {
        frequency: mix_freq,
        format: map_audio_format(mix_format),
        channels: mix_channels,
        group,
    };

    Ok(Sound {
        opl: Arc::new(Mutex::new(opl)),
        mix_config: Arc::new(Mutex::new(mix_config)),
        modes: Arc::new(Mutex::new(default_modes())),
        rt,
        sound_playing: Arc::new(Mutex::new(None)),
        left_pos: 0,
        right_pos: 0,
    })
}

#[cfg(feature = "web")]
pub async fn startup(_: Arc<Runtime>) -> Result<Sound, String> {
    //"TODO impl proper web sd startup (currently a dummy)")
    let mut opl = OPL::new().await?;
    opl.init(OPL_SETTINGS).await?;

    let digi_context = init_digi_sound_context()?;

    Ok(Sound {
        modes: default_modes(),
        opl: opl,
        sound_playing: None,
        left_pos: 0,
        right_pos: 0,
        digi_context,
    })
}

#[cfg(feature = "web")]
fn init_digi_sound_context() -> Result<AudioContext, String> {
    let opts = AudioContextOptions::new();
    opts.set_sample_rate(TARGET_SAMPLE_RATE);

    let ctx =
        AudioContext::new_with_context_options(&opts).map_err(|_| "digi audio context init")?;
    Ok(ctx)
}

#[cfg(feature = "sdl")]
impl Sound {
    pub fn is_sound_playing(&mut self, sound: SoundName) -> bool {
        let playing_mon = self.sound_playing.lock().unwrap();
        if let Some(playing_sound) = *playing_mon {
            playing_sound == sound
        } else {
            false
        }
    }

    pub fn is_any_sound_playing(&mut self) -> bool {
        let playing_mon = self.sound_playing.lock().unwrap();
        playing_mon.is_some()
    }

    pub fn force_play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        {
            let mut playing = self.sound_playing.lock().unwrap();
            *playing = Some(sound); // This sound _will_ be played
        }

        let modes = {
            let mode_mon = self.modes.lock().unwrap();
            *mode_mon
        };

        let may_digi_sound = assets.digi_sounds.get(&sound);
        if may_digi_sound.is_some() && modes.digi == DigiMode::SoundBlaster {
            let digi_sound = may_digi_sound.expect("some digi sound");
            let data_clone = digi_sound.chunk.clone();
            let mut channel = self.get_channel_for_digi(digi_sound.channel);
            channel.halt();
            self.set_position(&mut channel).expect("set sound position");
            let playing_mutex = self.sound_playing.clone();
            self.rt.spawn_blocking(move || {
                let chunk = mixer::Chunk::from_raw_buffer(data_clone).expect("chunk");
                channel.play(&chunk, 0).expect("play digi sound");
                // TODO inefficient. Only exists to keep the chunk referenced and not collected
                // Real fix would be to make Chunk in SDL sync so that this works properly and
                // the chunk can be prepared in the digi sound setup.
                while channel.is_playing() {
                    sleep(Duration::from_millis(1));
                }

                let mut m = playing_mutex.lock().unwrap();
                *m = None
            });
        } else {
            if modes.sound == SoundMode::AdLib {
                let adl_sound = assets.audio_sounds[sound as usize].clone();
                let playing_mutex = self.sound_playing.clone();
                let opl_mutex = self.opl.clone();
                {
                    // abort the currently playing sound (if any)
                    let mut opl = opl_mutex.lock().unwrap();
                    opl.stop_adl().expect("stop adl");
                }

                self.rt.spawn_blocking(move || {
                    {
                        let mut opl = opl_mutex.lock().unwrap();
                        opl.play_adl(adl_sound).expect("play sound file");
                    }
                    let mut is_playing = true;
                    while is_playing {
                        sleep(Duration::from_millis(1));
                        let mut opl = opl_mutex.lock().unwrap();
                        is_playing = opl.is_adl_playing().expect("playing state");
                    }
                    let mut m = playing_mutex.lock().unwrap();
                    *m = None
                });
            }
        }

        true
    }

    fn set_position(&mut self, channel: &mut Channel) -> Result<(), String> {
        if self.left_pos > 15
            || self.right_pos > 15
            || (self.left_pos == 15 && self.right_pos == 15)
        {
            quit(Some("set_position: Illegal position"));
        }

        channel.set_panning(
            ((15 - self.left_pos) << 4) + 15,
            ((15 - self.right_pos) << 4) + 15,
        )?;

        // reset to default for next sound
        self.left_pos = 0;
        self.right_pos = 0;

        Ok(())
    }

    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        // check priority
        {
            let playing = self.sound_playing.lock().unwrap();
            if !check_sound_prio(&*playing, assets, sound) {
                return false;
            }
        }

        self.force_play_sound(sound, assets)
    }

    pub fn play_music(&mut self, track: Music, assets: &Assets, loader: &dyn Loader) {
        if self.modes.lock().unwrap().music == MusicMode::Off {
            return;
        }

        let track_data = load_track(track, assets, loader);

        let mut opl_mon = self.opl.lock().unwrap();
        opl_mon.play_imf(track_data).expect("play imf")
    }

    fn get_channel_for_digi(&self, channel: DigiChannel) -> Channel {
        match channel {
            DigiChannel::Any => {
                let mon = self.mix_config.lock().unwrap();
                if let Some(ch) = mon.group.find_available() {
                    ch
                } else if let Some(ch) = mon.group.find_oldest() {
                    ch
                } else {
                    Channel::all()
                }
            }
            DigiChannel::Player => Channel(0),
            DigiChannel::Boss => Channel(1),
        }
    }

    pub fn play_sound_loc_tile(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        rc: &RayCast,
        tile_x: usize,
        tile_y: usize,
    ) {
        let gx = Fixed::new_from_i32(((tile_x as i32) << TILESHIFT) + (1 << (TILESHIFT - 1)));
        let gy = Fixed::new_from_i32(((tile_y as i32) << TILESHIFT) + (1 << (TILESHIFT - 1)));
        self.play_sound_loc_global(sound, assets, rc, gx, gy);
    }

    pub fn play_sound_loc_actor(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        rc: &RayCast,
        obj: &ObjType,
    ) {
        self.play_sound_loc_global(
            sound,
            assets,
            rc,
            Fixed::new_from_i32(obj.x),
            Fixed::new_from_i32(obj.y),
        );
    }

    fn play_sound_loc_global(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        rc: &RayCast,
        gx: Fixed,
        gy: Fixed,
    ) {
        let (left, right) = sound_loc(rc, gx, gy);
        self.left_pos = left;
        self.right_pos = right;
        self.play_sound(sound, assets);
    }

    pub fn prepare_digi_sound(
        &self,
        channel: DigiChannel,
        original_data: Vec<u8>,
    ) -> Result<DigiSound, String> {
        let mon = self.mix_config.lock().unwrap();
        let cvt = AudioCVT::new(
            audio::AudioFormat::U8,
            1,
            SOURCE_SAMPLE_RATE,
            mon.format,
            mon.channels as u8,
            mon.frequency,
        )?;

        let converted_data = cvt.convert(original_data);
        let boxed = converted_data.into_boxed_slice();
        Ok(DigiSound {
            chunk: boxed,
            channel,
        })
    }

    pub fn sound_mode(&self) -> SoundMode {
        let mon = self.modes.lock().unwrap();
        mon.sound
    }

    pub fn set_sound_mode(&mut self, mode: SoundMode) {
        let mut mon = self.modes.lock().unwrap();
        mon.sound = mode;
    }

    pub fn digi_mode(&self) -> DigiMode {
        let mon = self.modes.lock().unwrap();
        mon.digi
    }

    pub fn set_digi_mode(&mut self, mode: DigiMode) {
        let mut mon = self.modes.lock().unwrap();
        mon.digi = mode;
    }

    pub fn music_mode(&self) -> MusicMode {
        let mon = self.modes.lock().unwrap();
        mon.music
    }

    pub fn set_music_mode(&mut self, mode: MusicMode) {
        let mut mode_mon = self.modes.lock().expect("mode lock");
        mode_mon.music = mode;
        if mode == MusicMode::Off {
            let mut opl_mon = self.opl.lock().expect("opl lock");
            clear_music(&mut opl_mon).expect("clear music");
        }
    }
}

#[cfg(any(feature = "sdl", feature = "test"))]
fn map_audio_format(format: mixer::AudioFormat) -> AudioFormat {
    match format {
        mixer::AUDIO_S16LSB => AudioFormat::S16LSB,
        _ => todo!("impl mapping"),
    }
}

fn clear_music(opl: &mut OPL) -> Result<(), String> {
    opl.stop_imf()?;
    opl.write_reg(0xBD, 0)?;
    for i in 0..MAX_TRACKS as u32 {
        opl.write_reg(0xB0 + i + 1, 0)?;
    }
    Ok(())
}

fn load_track(track: Music, assets: &Assets, loader: &dyn Loader) -> Vec<u8> {
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

fn check_sound_prio(playing_sound: &Option<SoundName>, assets: &Assets, sound: SoundName) -> bool {
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

fn sound_loc(rc: &RayCast, gx_param: Fixed, gy_param: Fixed) -> (u8, u8) {
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

#[cfg(feature = "web")]
impl Sound {
    pub fn is_sound_playing(&mut self, sound: SoundName) -> bool {
        if let Some(playing_sound) = self.sound_playing {
            playing_sound == sound
        } else {
            false
        }
    }

    pub fn is_any_sound_playing(&mut self) -> bool {
        self.sound_playing.is_some()
    }

    pub fn force_play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        self.sound_playing = Some(sound); // This sound _will_ be played

        let may_digi_sound = assets.digi_sounds.get(&sound);
        if let Some(digi_sound) = may_digi_sound
            && self.modes.digi == DigiMode::SoundBlaster
        {
            self.play_digi(digi_sound).expect("play digi sound")
        } else {
            if self.modes.sound == SoundMode::AdLib {
                let adl_sound = assets.audio_sounds[sound as usize].clone();
                self.opl.play_adl(adl_sound).expect("play adl sound");
            }
        }
        true
    }

    fn play_digi(&self, digi_sound: &DigiSound) -> Result<(), &str> {
        let frames = digi_sound.chunk.len() as u32;
        let buffer = self
            .digi_context
            .create_buffer(1, frames, TARGET_SAMPLE_RATE)
            .map_err(|_| "create audio buffer")?;
        buffer
            .copy_to_channel_with_start_in_channel(&digi_sound.chunk, 0, 0)
            .map_err(|_| "data copy to channel")?;

        let src = self
            .digi_context
            .create_buffer_source()
            .map_err(|_| "buffer source creation")?;
        src.set_buffer(Some(&buffer));
        src.playback_rate().set_value(PLAYBACK_RATE);

        src.connect_with_audio_node(&self.digi_context.destination())
            .map_err(|_| "audio connect")?;

        src.start().map_err(|_| "sound start")
    }

    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        /*
        TODO check sound prio on web, for that we need to be able
        to track the sound end in OPL and the digi audio context
        if !check_sound_prio(&self.sound_playing, assets, sound) {
            return false;
        }
        */
        self.force_play_sound(sound, assets)
    }

    pub fn play_music(&mut self, track: Music, assets: &Assets, loader: &dyn Loader) {
        if self.modes.music == MusicMode::Off {
            return;
        }

        let track_data = load_track(track, assets, loader);
        self.opl.play_imf(track_data).expect("play imf")
    }

    pub fn play_sound_loc_tile(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        rc: &RayCast,

        tile_x: usize,
        tile_y: usize,
    ) {
        let gx = Fixed::new_from_i32(((tile_x as i32) << TILESHIFT) + (1 << (TILESHIFT - 1)));
        let gy = Fixed::new_from_i32(((tile_y as i32) << TILESHIFT) + (1 << (TILESHIFT - 1)));
        self.play_sound_loc_global(sound, assets, rc, gx, gy);
    }

    pub fn play_sound_loc_actor(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        rc: &RayCast,
        obj: &ObjType,
    ) {
        self.play_sound_loc_global(
            sound,
            assets,
            rc,
            Fixed::new_from_i32(obj.x),
            Fixed::new_from_i32(obj.y),
        );
    }

    fn play_sound_loc_global(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        rc: &RayCast,
        gx: Fixed,
        gy: Fixed,
    ) {
        let (left, right) = sound_loc(rc, gx, gy);
        self.left_pos = left;
        self.right_pos = right;
        self.play_sound(sound, assets);
    }

    pub fn prepare_digi_sound(
        &self,
        _: DigiChannel,
        original_data: Vec<u8>,
    ) -> Result<DigiSound, String> {
        let converted: Vec<f32> = original_data
            .iter()
            .map(|&s| (s as f32 - 128.0) / 128.0)
            .collect();
        Ok(DigiSound { chunk: converted })
    }

    pub fn sound_mode(&self) -> SoundMode {
        self.modes.sound
    }

    pub fn set_sound_mode(&mut self, mode: SoundMode) {
        self.modes.sound = mode;
    }

    pub fn digi_mode(&self) -> DigiMode {
        self.modes.digi
    }

    pub fn set_digi_mode(&mut self, mode: DigiMode) {
        self.modes.digi = mode;
    }

    pub fn music_mode(&self) -> MusicMode {
        self.modes.music
    }

    pub fn set_music_mode(&mut self, mode: MusicMode) {
        self.modes.music = mode;
        if mode == MusicMode::Off {
            clear_music(&mut self.opl).expect("clear music");
        }
    }
}

#[cfg(feature = "test")]
impl Sound {
    pub fn is_sound_playing(&mut self, _sound: SoundName) -> bool {
        true
    }

    pub fn is_any_sound_playing(&mut self) -> bool {
        false
    }

    pub fn force_play_sound(&mut self, _sound: SoundName, _assets: &Assets) -> bool {
        true
    }

    pub fn play_sound(&mut self, _sound: SoundName, _assets: &Assets) -> bool {
        true
    }

    pub fn play_music(&mut self, _track: Music, _assets: &Assets, _loader: &dyn Loader) {
        // do nothing
    }

    pub fn play_sound_loc_tile(
        &mut self,
        _sound: SoundName,
        _assets: &Assets,
        _rc: &RayCast,
        _tile_x: usize,
        _tile_y: usize,
    ) {
        // do nothing
    }

    pub fn play_sound_loc_actor(
        &mut self,
        _sound: SoundName,
        _assets: &Assets,
        _rc: &RayCast,
        _obj: &ObjType,
    ) {
        // do nothing
    }

    fn play_sound_loc_global(
        &mut self,
        _sound: SoundName,
        _assets: &Assets,
        _rc: &RayCast,
        _tile_x: usize,
        _tile_y: usize,
    ) {
        // do nothing
    }

    pub fn prepare_digi_sound(
        &self,
        _channel: DigiChannel,
        _original_data: Vec<u8>,
    ) -> Result<DigiSound, String> {
        Ok(DigiSound {
            chunk: Box::new([0; 0]),
            channel: DigiChannel::Any,
        })
    }

    pub fn sound_mode(&self) -> SoundMode {
        SoundMode::Off
    }

    pub fn set_sound_mode(&mut self, _mode: SoundMode) {
        // mode can't be changed in test
    }

    pub fn digi_mode(&self) -> DigiMode {
        DigiMode::Off
    }

    pub fn set_digi_mode(&mut self, _mode: DigiMode) {
        // digi_mode can't be changed in test
    }

    pub fn music_mode(&self) -> MusicMode {
        MusicMode::Off
    }

    pub fn set_music_mode(&mut self, _mode: MusicMode) {
        // music_mode can't be changed in test
    }
}
