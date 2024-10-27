use crate::{
    assets::SoundName,
    def::{Assets, DigiSound, ObjType, TILESHIFT},
};

#[cfg(feature = "sdl")]
use async_std::task::spawn;

use opl::OPL;

#[cfg(feature = "sdl")]
use sdl2::audio::{self, AudioCVT, AudioFormat};
#[cfg(feature = "sdl")]
use sdl2::mixer::{self};

const ORIG_SAMPLE_RATE: i32 = 7042;

#[cfg(feature = "sdl")]
pub struct DigiMixConfig {
    pub frequency: i32,
    pub format: AudioFormat,
    pub channels: i32,
}

pub struct DigiInfo {
    pub start_page: usize,
    pub length: usize,
}

#[cfg(feature = "sdl")]
pub struct Sound {
    pub opl: OPL,
    pub mix_config: DigiMixConfig,
}

#[cfg(feature = "web")]
pub struct Sound {
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
    };

    Ok(Sound { opl, mix_config })
}

#[cfg(feature = "web")]
pub fn startup() -> Result<Sound, String> {
    todo!("impl web sd startup")
}

#[cfg(feature = "sdl")]
impl Sound {
    pub fn play_sound(&mut self, sound: SoundName, assets: &Assets) {
        if let Some(digi_sound) = assets.digi_sounds.get(&sound) {
            let data_clone = digi_sound.chunk.clone();
            spawn(async {
                let chunk = mixer::Chunk::from_raw_buffer(data_clone).expect("chunk");
                let channel = mixer::Channel::all();
                channel.play(&chunk, 0).expect("play digi sound test");
                // TODO inefficient. Only exists to keep the chunk referenced and not collected
                // Real fix would be to make Chunk in SDL sync so that this works properly and
                // the chunk can be prepared in the digi sound setup.
                while channel.is_playing() {}
            });
        } else {
            let sound = &assets.audio_sounds[sound as usize];
            self.opl.play_adl(sound.clone()).expect("play sound file");
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

    pub fn prepare_digi_sound(&self, original_data: Vec<u8>) -> Result<DigiSound, String> {
        let cvt = AudioCVT::new(
            audio::AudioFormat::U8,
            1,
            ORIG_SAMPLE_RATE,
            self.mix_config.format,
            self.mix_config.channels as u8,
            self.mix_config.frequency,
        )?;

        let converted_data = cvt.convert(original_data);
        let boxed = converted_data.into_boxed_slice();
        Ok(DigiSound { chunk: boxed })
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

    #[cfg(feature = "web")]
    pub fn prepare_digi_sound(&self, original_data: Vec<u8>) -> Result<DigiSound, String> {
        todo!("impl web digi sound preparation")
    }
}
