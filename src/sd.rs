use crate::{assets::SoundName, def::Assets};

use opl::OPL;

pub struct Sound {
    pub opl: OPL,
}

impl Sound {
    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) {
        let sound = &assets.audio_sounds[sound as usize];
        self.opl.play_adl(sound.clone()).expect("play sound file");
    }
}
