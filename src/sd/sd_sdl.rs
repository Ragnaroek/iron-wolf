use sdl2::audio::{self, AudioCVT, AudioFormat};
use sdl2::mixer::{self, Channel};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::Runtime;

use opl::{OPL, OPLSettings};

use crate::assets::{DigiChannel, Music, SoundName};
use crate::def::{Assets, ObjType, TILESHIFT};
use crate::draw::RayCast;
use crate::fixed::Fixed;
use crate::loader::Loader;
use crate::sd::{
    DigiMode, Modes, MusicMode, SOURCE_SAMPLE_RATE, SoundMode, check_sound_prio, clear_music,
    default_modes, load_track, sound_loc,
};
use crate::start::quit;

pub struct DigiSound {
    pub chunk: Box<[u8]>,
    pub channel: DigiChannel,
}

pub struct DigiMixConfig {
    pub frequency: i32,
    pub format: AudioFormat,
    pub channels: i32,
    pub group: mixer::Group,
}

pub struct Sound {
    modes: Arc<Mutex<Modes>>,
    opl: Arc<Mutex<OPL>>,
    mix_config: Arc<Mutex<DigiMixConfig>>,
    rt: Arc<Runtime>,
    sound_playing: Arc<Mutex<Option<SoundName>>>,
    left_pos: u8,
    right_pos: u8,
}

const OPL_SETTINGS: OPLSettings = OPLSettings {
    mixer_rate: 49716,
    imf_clock_rate: 700,
    adl_clock_rate: 140,
};

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

    pub fn play_music(&mut self, track: Music, assets: &Assets, loader: &Loader) {
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

fn map_audio_format(format: mixer::AudioFormat) -> AudioFormat {
    match format {
        mixer::AUDIO_S16LSB => AudioFormat::S16LSB,
        _ => todo!("impl mapping"),
    }
}
