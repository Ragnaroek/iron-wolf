#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;

use crate::{assets::WolfFile, loader::Loader};
use std::fs;
use std::path::Path;

use super::def::IWConfig;
use super::user;
use super::util;

use vga::input::{NumCode, to_numcode};

pub const IW_CONFIG_FILE_NAME: &str = "iw_config.toml";
pub const CONFIG_DATA: &'static str = "CONFIG.WL6";
pub const MAX_HIGH_NAME: usize = 57;
pub const MAX_SCORES: usize = 7;

// Load the config from the config file if it exists.
// Returns the default config (vanila mode) if no config
// file can be found.
// Checks the current working dir for the presence of a
// iw_config.toml file.
pub fn read_iw_config() -> Result<IWConfig, String> {
    let conf_file = Path::new(IW_CONFIG_FILE_NAME);
    if conf_file.exists() {
        let content = fs::read_to_string(conf_file).map_err(|e| e.to_string())?;
        let config: IWConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        default_iw_config()
    }
}

pub fn default_iw_config() -> Result<IWConfig, String> {
    toml::from_str("vanilla = true").map_err(|e| e.to_string())
}

#[derive(Copy, Clone)]
pub enum SDMode {
    Off = 0,
    PC = 1,
    AdLib = 2,
}

#[derive(Copy, Clone)]
pub enum SMMode {
    Off = 0,
    AdLib = 1,
}

#[derive(Copy, Clone)]
pub enum SDSMode {
    Off = 0,
    PC = 1,
    SoundSource = 2,
    SoundBlaster = 3,
}

// the original Wolf3D Config
pub struct WolfConfig {
    pub high_scores: Vec<user::HighScore>,

    pub sd_mode: SDMode,
    pub sm_mode: SMMode,
    pub sds_mode: SDSMode,

    pub mouse_enabled: bool,
    pub joystick_enabled: bool,
    pub joypad_enabled: bool,
    pub joystick_progressive: u16,
    pub joystick_port: u16,

    pub dirscan: [NumCode; 4],
    pub buttonscan: [NumCode; 8],
    pub buttonmouse: [NumCode; 4],
    pub buttonjoy: [NumCode; 4],

    pub viewsize: u16,
    pub mouse_adjustment: u16,
}

pub fn write_wolf_config(loader: &dyn Loader, wolf_config: &WolfConfig) -> Result<(), String> {
    let mut writer = util::new_data_writer(522); // TODO always 522 bytes?

    for i in 0..MAX_SCORES {
        let high_score = &wolf_config.high_scores[i];
        writer.write_utf8_string(&high_score.name, MAX_HIGH_NAME + 1);
        writer.write_u32(high_score.score);
        writer.write_u16(high_score.completed);
        writer.write_u16(high_score.episode);
    }

    writer.write_u16(wolf_config.sd_mode as u16);
    writer.write_u16(wolf_config.sm_mode as u16);
    writer.write_u16(wolf_config.sds_mode as u16);

    writer.write_bool(wolf_config.mouse_enabled);
    writer.write_bool(wolf_config.joystick_enabled);
    writer.write_bool(wolf_config.joypad_enabled);
    writer.write_u16(wolf_config.joystick_progressive);
    writer.write_u16(wolf_config.joystick_port);

    for i in 0..4 {
        writer.write_u16(numcode_to_u16(wolf_config.dirscan[i]));
    }
    for i in 0..8 {
        writer.write_u16(numcode_to_u16(wolf_config.buttonscan[i]));
    }
    for i in 0..4 {
        writer.write_u16(numcode_to_u16(wolf_config.buttonmouse[i]));
    }
    for i in 0..4 {
        writer.write_u16(numcode_to_u16(wolf_config.buttonjoy[i]));
    }

    writer.write_u16(wolf_config.viewsize);
    writer.write_u16(wolf_config.mouse_adjustment);

    loader.write_wolf_file(WolfFile::ConfigData, &writer.data)
}

pub fn numcode_to_u16(code: NumCode) -> u16 {
    if code == NumCode::Bad {
        0xFFFF
    } else {
        code as u16
    }
}

pub fn load_wolf_config(loader: &dyn Loader) -> WolfConfig {
    let data = loader.load_wolf_file(WolfFile::ConfigData);
    let mut reader = util::new_data_reader(&data);

    let mut high_scores = Vec::with_capacity(MAX_SCORES);

    for _ in 0..MAX_SCORES {
        let mut name = reader.read_utf8_string(MAX_HIGH_NAME + 1);
        name.retain(|c| c != '\0');

        let score = reader.read_u32();
        let completed = reader.read_u16();
        let episode = reader.read_u16();

        high_scores.push(user::HighScore {
            name,
            score,
            completed,
            episode,
        });
    }

    let sd_mode = sd_mode(reader.read_u16());
    let sm_mode = sm_mode(reader.read_u16());
    let sds_mode = sds_mode(reader.read_u16());

    let mouse_enabled = reader.read_bool();
    let joystick_enabled = reader.read_bool();
    let joypad_enabled = reader.read_bool();
    let joystick_progressive = reader.read_u16();
    let joystick_port = reader.read_u16();

    let mut dirscan = [NumCode::None; 4];
    for i in 0..4 {
        dirscan[i] = to_numcode(reader.read_u16() as u8);
    }
    let mut buttonscan = [NumCode::None; 8];
    for i in 0..8 {
        buttonscan[i] = to_numcode(reader.read_u16() as u8);
    }
    let mut buttonmouse = [NumCode::None; 4];
    for i in 0..4 {
        buttonmouse[i] = to_numcode(reader.read_u16() as u8);
    }
    let mut buttonjoy = [NumCode::None; 4];
    for i in 0..4 {
        buttonjoy[i] = to_numcode(reader.read_u16() as u8);
    }
    let viewsize = reader.read_u16();
    let mouse_adjustment = reader.read_u16();

    WolfConfig {
        high_scores,
        sd_mode,
        sm_mode,
        sds_mode,
        mouse_enabled,
        joystick_enabled,
        joypad_enabled,
        joystick_progressive,
        joystick_port,
        dirscan,
        buttonscan,
        buttonmouse,
        buttonjoy,
        viewsize,
        mouse_adjustment,
    }
}

fn sd_mode(v: u16) -> SDMode {
    match v {
        0 => return SDMode::Off,
        1 => return SDMode::PC,
        2 => return SDMode::AdLib,
        _ => return SDMode::Off,
    }
}

fn sm_mode(v: u16) -> SMMode {
    match v {
        0 => return SMMode::Off,
        1 => return SMMode::AdLib,
        _ => return SMMode::Off,
    }
}

fn sds_mode(v: u16) -> SDSMode {
    match v {
        0 => return SDSMode::Off,
        1 => return SDSMode::PC,
        2 => return SDSMode::SoundSource,
        3 => return SDSMode::SoundBlaster,
        _ => return SDSMode::Off,
    }
}
