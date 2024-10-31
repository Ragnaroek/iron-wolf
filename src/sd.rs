use std::sync::{Arc, Mutex};

use crate::{
    assets::{DigiChannel, SoundName},
    def::{Assets, DigiSound, ObjType, TILESHIFT},
};

#[cfg(feature = "sdl")]
use async_std::task::spawn;

use opl::OPL;

#[cfg(feature = "sdl")]
use sdl2::audio::{self, AudioCVT, AudioFormat};
#[cfg(feature = "sdl")]
use sdl2::mixer::{self, Channel};

const ORIG_SAMPLE_RATE: i32 = 7042;
const MAX_TRACKS: usize = 10;

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

struct Modes {
    sound: SoundMode,
    digi: DigiMode,
    music: MusicMode,
}

fn default_modes() -> Arc<Mutex<Modes>> {
    Arc::new(Mutex::new(Modes {
        sound: SoundMode::AdLib,
        digi: DigiMode::SoundBlaster,
        music: MusicMode::AdLib,
    }))
}

pub struct DigiInfo {
    pub start_page: usize,
    pub length: usize,
}

#[cfg(feature = "sdl")]
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
}

#[cfg(feature = "web")]
pub struct Sound {
    modes: Modes,
    pub opl: OPL,
}

#[cfg(feature = "sdl")]
pub fn startup() -> Result<Sound, String> {
    let mut opl = opl::new()?;
    opl.init(opl::OPLSettings {
        mixer_rate: 49716,
        imf_clock_rate: 700,
        adl_clock_rate: 140,
    });

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
        modes: default_modes(),
    })
}

#[cfg(feature = "web")]
pub fn startup() -> Result<Sound, String> {
    todo!("impl web sd startup")
}

#[cfg(feature = "sdl")]
impl Sound {
    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) {
        let mode_mon = self.modes.lock().unwrap();
        let may_digi_sound = assets.digi_sounds.get(&sound);
        if may_digi_sound.is_some() && mode_mon.digi == DigiMode::SoundBlaster {
            let digi_sound = may_digi_sound.expect("some digi sound");
            let data_clone = digi_sound.chunk.clone();
            let channel = self.get_channel_for_digi(digi_sound.channel);
            spawn(async move {
                let chunk = mixer::Chunk::from_raw_buffer(data_clone).expect("chunk");
                channel.play(&chunk, 0).expect("play digi sound");
                // TODO inefficient. Only exists to keep the chunk referenced and not collected
                // Real fix would be to make Chunk in SDL sync so that this works properly and
                // the chunk can be prepared in the digi sound setup.
                while channel.is_playing() {}
            });
        } else {
            if mode_mon.sound == SoundMode::AdLib {
                let sound = &assets.audio_sounds[sound as usize];
                let mut mon = self.opl.lock().unwrap();
                mon.play_adl(sound.clone()).expect("play sound file");
            }
        }
    }

    pub fn play_imf(&mut self, data: Vec<u8>) -> Result<(), &'static str> {
        if self.modes.lock().unwrap().music == MusicMode::Off {
            return Ok(());
        }

        let mut opl_mon = self.opl.lock().unwrap();
        opl_mon.play_imf(data)
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
        tile_x: usize,
        tile_y: usize,
    ) {
        self.play_sound_loc_global(
            sound,
            assets,
            (tile_x << TILESHIFT) + (1 << (TILESHIFT - 1)),
            (tile_y << TILESHIFT) + (1 << (TILESHIFT - 1)),
        );
    }

    pub fn play_sound_loc_actor(&mut self, sound: SoundName, assets: &Assets, obj: &ObjType) {
        self.play_sound_loc_global(sound, assets, obj.x as usize, obj.y as usize);
    }

    fn play_sound_loc_global(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        tile_x: usize,
        tile_y: usize,
    ) {
        // TODO set sound location and position for digitized sounds
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
            ORIG_SAMPLE_RATE,
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
            opl_mon.stop_imf().expect("stop music");
            opl_mon.write_reg(0xBD, 0).expect("flush effects");
            for i in 0..MAX_TRACKS as u32 {
                opl_mon.write_reg(0xB0 + i + 1, 0).expect("flush freq");
            }
        }
    }
}

#[cfg(feature = "sdl")]
fn map_audio_format(format: mixer::AudioFormat) -> AudioFormat {
    match format {
        mixer::AUDIO_S16LSB => AudioFormat::S16LSB,
        _ => todo!("impl mapping"),
    }
}

#[cfg(feature = "web")]
impl Sound {
    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) {
        todo!("impl play sound web");
    }

    pub fn play_imf(&mut self, data: Vec<u8>) -> Result<(), &'static str> {
        todo!("impl play imf web");
    }

    pub fn play_sound_loc_tile(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        tile_x: usize,
        tile_y: usize,
    ) {
        todo!("impl play sound loc tile web");
    }

    pub fn play_sound_loc_actor(&mut self, sound: SoundName, assets: &Assets, obj: &ObjType) {
        todo!("impl play sound actor web");
    }

    fn play_sound_loc_global(
        &mut self,
        sound: SoundName,
        assets: &Assets,
        tile_x: usize,
        tile_y: usize,
    ) {
        todo!("impl play sound loc global web");
    }

    pub fn prepare_digi_sound(
        &self,
        channel: DigiChannel,
        original_data: Vec<u8>,
    ) -> Result<DigiSound, String> {
        todo!("impl web digi sound preparation")
    }

    pub fn sound_mode(&self) -> SoundMode {
        todo!("sound mode web")
    }

    pub fn set_sound_mode(&mut self, mode: SoundMode) {
        todo!("set sound mode web")
    }

    pub fn digi_mode(&self) -> DigiMode {
        todo!("digi mode web")
    }

    pub fn set_digi_mode(&mut self, mode: DigiMode) {
        todo!("set digi mode web")
    }

    pub fn music_mode(&self) -> MusicMode {
        todo!("music mode web")
    }

    pub fn set_music_mode(&mut self, mode: MusicMode) {
        todo!("set music mode web")
    }
}
