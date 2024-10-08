use crate::{
    assets::SoundName,
    def::{Assets, ObjType, TILESHIFT},
};

use opl::OPL;

pub struct Sound {
    pub opl: OPL,
}

impl Sound {
    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) {
        let sound = &assets.audio_sounds[sound as usize];
        self.opl.play_adl(sound.clone()).expect("play sound file");
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
}
