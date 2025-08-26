#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;

use crate::def::{Button, NUM_BUTTONS, NUM_MOUSE_BUTTONS};
use crate::util::DataReader;
use crate::{assets::WolfFile, loader::Loader};
use std::env;
use std::fs;
use std::path::Path;

use super::def::IWConfig;
use super::user;
use super::util::DataWriter;

use vga::input::{NumCode, to_numcode};

pub const IW_CONFIG_FILE_NAME: &str = "iw_config.toml";
pub const CONFIG_DATA: &'static str = "CONFIG.WL6";
pub const MAX_HIGH_NAME: usize = 57;
pub const MAX_SCORES: usize = 7;

// Load the config from the config file if it exists.
// Returns the default config (vanila mode) if no config
// file can be found.
// Checks first the arguments for a config file and after that
// the current working dir for the presence of a
// iw_config.toml file.
pub fn read_iw_config() -> Result<IWConfig, String> {
    if let Some(conf_env) = check_config_env() {
        let path = Path::new(&conf_env);
        return read_conf_file(&path);
    }

    let conf_file = Path::new(IW_CONFIG_FILE_NAME);
    if conf_file.exists() {
        read_conf_file(conf_file)
    } else {
        default_iw_config()
    }
}

fn read_conf_file(conf_file: &Path) -> Result<IWConfig, String> {
    let content = fs::read_to_string(conf_file).map_err(|e| e.to_string())?;
    let config: IWConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config)
}

pub fn default_iw_config() -> Result<IWConfig, String> {
    toml::from_str("vanilla = true").map_err(|e| e.to_string())
}

fn check_config_env() -> Option<String> {
    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg == "-config" {
            return args.next();
        }
    }
    None
}

pub fn check_timedemo_env() -> Option<usize> {
    let mut args = env::args();
    while let Some(arg) = args.next() {
        if arg == "-timedemo" {
            let may_which = args.next();
            if let Some(which) = may_which {
                return Some(which.parse().expect("demo number"));
            } else {
                panic!("-timedemo needs the demo number parameter")
            }
        }
    }
    None
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

    pub dir_scan: [NumCode; 4],
    pub button_scan: [NumCode; NUM_BUTTONS],
    pub button_mouse: [Button; NUM_MOUSE_BUTTONS],
    pub button_joy: [NumCode; 4],

    pub viewsize: u16,
    pub mouse_adjustment: u16,
}

pub fn write_wolf_config(loader: &dyn Loader, wolf_config: &WolfConfig) -> Result<(), String> {
    let mut writer = DataWriter::new(522);

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
        writer.write_u16(numcode_to_u16(wolf_config.dir_scan[i]));
    }
    for i in 0..NUM_BUTTONS {
        writer.write_u16(numcode_to_u16(wolf_config.button_scan[i]));
    }
    for i in 0..NUM_MOUSE_BUTTONS {
        writer.write_u16(button_to_u16(wolf_config.button_mouse[i]));
    }
    for i in 0..4 {
        writer.write_u16(numcode_to_u16(wolf_config.button_joy[i]));
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

fn button_to_u16(button: Button) -> u16 {
    if button == Button::NoButton {
        0xFFFF
    } else {
        button as u16
    }
}

pub fn load_wolf_config(loader: &dyn Loader) -> WolfConfig {
    let data = loader.load_wolf_file(WolfFile::ConfigData);
    let mut reader = DataReader::new(&data);

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

    let mut dir_scan = [NumCode::None; 4];
    for i in 0..4 {
        dir_scan[i] = to_numcode(reader.read_u16() as u8);
    }
    let mut button_scan = [NumCode::None; NUM_BUTTONS];
    for i in 0..NUM_BUTTONS {
        button_scan[i] = to_numcode(reader.read_u16() as u8);
    }
    let mut button_mouse = [Button::NoButton; NUM_MOUSE_BUTTONS];
    for i in 0..NUM_MOUSE_BUTTONS {
        button_mouse[i] = Button::from_usize(reader.read_u16() as usize);
    }
    let mut button_joy = [NumCode::None; 4];
    for i in 0..4 {
        button_joy[i] = to_numcode(reader.read_u16() as u8);
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
        dir_scan,
        button_scan,
        button_mouse,
        button_joy,
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
