use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use web_sys::{AudioContext, AudioContextOptions};

use opl::{OPL, OPLSettings};

use crate::assets::{DigiChannel, Music, SoundName};
use crate::def::{Assets, DigiSound, ObjType, TILESHIFT};
use crate::draw::RayCast;
use crate::fixed::Fixed;
use crate::loader::Loader;
use crate::sd::{
    DigiMode, Modes, MusicMode, SOURCE_SAMPLE_RATE, SoundMode, check_sound_prio, clear_music,
    default_modes, load_track, sound_loc,
};
use crate::start::quit;

pub struct Sound {
    modes: Modes,
    pub opl: OPL,
    sound_playing: Option<SoundName>,
    left_pos: u8,
    right_pos: u8,
    digi_context: AudioContext,
}

const OPL_SETTINGS: OPLSettings = OPLSettings {
    imf_clock_rate: 700,
    adl_clock_rate: 140,
};

const TARGET_SAMPLE_RATE: f32 = 44100.0;
const PLAYBACK_RATE: f32 = SOURCE_SAMPLE_RATE as f32 / TARGET_SAMPLE_RATE;

pub async fn startup(_: Arc<Runtime>) -> Result<Sound, String> {
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

fn init_digi_sound_context() -> Result<AudioContext, String> {
    let opts = AudioContextOptions::new();
    opts.set_sample_rate(TARGET_SAMPLE_RATE);

    let ctx =
        AudioContext::new_with_context_options(&opts).map_err(|_| "digi audio context init")?;
    Ok(ctx)
}

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
