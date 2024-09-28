use crate::{assets::SoundName, def::Assets};

use opl::OPL;

pub fn play_sound(sound: SoundName, opl: &mut OPL, assets: &Assets) {
    let sound = &assets.audio_sounds[sound as usize];
    opl.play_adl(sound.clone()).expect("play sound file");
}
