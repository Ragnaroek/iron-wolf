use std::sync::Arc;
use tokio::runtime::Runtime;

use opl::OPLSettings;

use crate::assets::{DigiChannel, Music, SoundName};
use crate::def::{Assets, ObjType};
use crate::draw::RayCast;
use crate::loader::Loader;
use crate::sd::{DigiMode, DigiSound, MusicMode, SoundMode};

pub struct Sound {}

pub fn startup(_rt: Arc<Runtime>) -> Result<Sound, String> {
    Ok(test_sound())
}

pub fn test_sound() -> Sound {
    Sound {}
}

const OPL_SETTINGS: OPLSettings = OPLSettings {
    mixer_rate: 49716,
    imf_clock_rate: 700,
    adl_clock_rate: 140,
};

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
