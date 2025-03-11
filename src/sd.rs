use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use tokio::runtime::Runtime;

use crate::{
    assets::{DigiChannel, Music, SoundName, WolfFile},
    def::{Assets, DigiSound, ObjType, TILESHIFT},
    loader::Loader,
};

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
    rt: Arc<Runtime>,
    sound_playing: Arc<Mutex<Option<SoundName>>>,
}

#[cfg(feature = "web")]
pub struct Sound {
    modes: Modes,
    pub opl: OPL,
}

#[cfg(feature = "sdl")]
pub fn startup(rt: Arc<Runtime>) -> Result<Sound, String> {
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
        rt,
        sound_playing: Arc::new(Mutex::new(None)),
    })
}

#[cfg(feature = "web")]
pub fn startup(_: Arc<Runtime>) -> Result<Sound, String> {
    todo!("impl web sd startup")
}

#[cfg(feature = "sdl")]
impl Sound {
    pub fn is_sound_playing(&mut self) -> Option<SoundName> {
        let playing_mon = self.sound_playing.lock().unwrap();
        *playing_mon
    }

    pub fn force_play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        {
            let mut playing = self.sound_playing.lock().unwrap();
            *playing = Some(sound); // This sound _will_ be played
        }

        let mode_mon = self.modes.lock().unwrap();

        let may_digi_sound = assets.digi_sounds.get(&sound);
        if may_digi_sound.is_some() && mode_mon.digi == DigiMode::SoundBlaster {
            let digi_sound = may_digi_sound.expect("some digi sound");
            let data_clone = digi_sound.chunk.clone();
            let channel = self.get_channel_for_digi(digi_sound.channel);
            channel.halt();
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
            if mode_mon.sound == SoundMode::AdLib {
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

    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        // check priority
        {
            let playing = self.sound_playing.lock().unwrap();
            if playing.is_some() {
                let playing_prio =
                    assets.audio_sounds[playing.expect("playing sound") as usize].priority;
                let new_sound_prio = assets.audio_sounds[sound as usize].priority;
                if new_sound_prio < playing_prio {
                    return false;
                }
            }
        }

        self.force_play_sound(sound, assets)
    }

    pub fn play_music(&mut self, track: Music, assets: &Assets, loader: &dyn Loader) {
        if self.modes.lock().unwrap().music == MusicMode::Off {
            return;
        }

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
    pub fn is_sound_playing(&mut self) -> Option<SoundName> {
        todo!("impl is_sound_playing for web");
    }

    pub fn force_play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        todo!("impl force play sound web");
    }

    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) -> bool {
        todo!("impl play sound web");
    }

    pub fn play_music(&mut self, track: Music, assets: &Assets, loader: &dyn Loader) {
        todo!("impl play music web");
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
