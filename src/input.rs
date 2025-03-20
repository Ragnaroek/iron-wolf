use vga::input::{self, NumCode};

use std::sync::{Arc, Mutex};

use crate::{config::WolfConfig, def::NUM_BUTTONS};

use super::time::{TimeCount, get_count};

#[derive(PartialEq)]
pub enum ControlDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    None,
}

pub struct ControlInfo {
    pub dir: ControlDirection,
}

pub struct Input {
    time: TimeCount,
    pub mouse_enabled: bool,
    pub joystick_enabled: bool,
    pub input_monitoring: Arc<Mutex<input::InputMonitoring>>,

    pub button_scan: [NumCode; NUM_BUTTONS],
    pub dir_scan: [NumCode; 4],
}

pub fn init(
    time: TimeCount,
    input_monitoring: Arc<Mutex<input::InputMonitoring>>,
    wolf_config: &WolfConfig,
) -> Input {
    Input {
        time,
        mouse_enabled: true,
        joystick_enabled: false,
        input_monitoring,
        button_scan: wolf_config.buttonscan.clone(),
        dir_scan: wolf_config.dirscan.clone(),
    }
}

impl Input {
    pub async fn wait_user_input(&self, delay: u64) -> bool {
        let last_count = get_count(&self.time);
        {
            let mut mon = self.input_monitoring.lock().unwrap();
            mon.clear_keyboard();
        }
        loop {
            if self.check_ack() {
                return true;
            }

            if get_count(&self.time) - last_count > delay {
                break;
            }
        }
        false
    }

    pub async fn ack(&self) -> bool {
        self.wait_user_input(u64::MAX).await
    }

    pub fn start_ack(&self) {
        let mut mon = self.input_monitoring.lock().unwrap();
        mon.clear_keyboard();
        // TODO clear mouse and joystick buttons
    }

    pub fn check_ack(&self) -> bool {
        let mon = self.input_monitoring.lock().unwrap();
        mon.any_key_pressed()
    }

    pub fn key_pressed(&self, code: NumCode) -> bool {
        let mon = self.input_monitoring.lock().unwrap();
        mon.key_pressed(code)
    }

    pub fn clear_keys_down(&self) {
        let mut mon = self.input_monitoring.lock().unwrap();
        mon.clear_keyboard();
        mon.keyboard.last_scan = NumCode::None;
        mon.keyboard.last_ascii = '\0';
    }

    pub fn clear_last_scan(&self) {
        let mut mon = self.input_monitoring.lock().unwrap();
        mon.keyboard.last_scan = NumCode::None;
    }

    pub fn last_scan(&self) -> NumCode {
        let mon = self.input_monitoring.lock().unwrap();
        mon.keyboard.last_scan
    }

    pub fn clear_last_ascii(&self) {
        let mut mon = self.input_monitoring.lock().unwrap();
        mon.keyboard.last_ascii = '\0';
    }

    // Returns the 0 char if nothing is set
    pub fn last_ascii(&self) -> char {
        let mon = self.input_monitoring.lock().unwrap();
        mon.keyboard.last_ascii
    }
}

pub fn read_control(input: &Input) -> ControlInfo {
    let dir;
    if input.key_pressed(NumCode::UpArrow) {
        dir = ControlDirection::North;
    } else if input.key_pressed(NumCode::DownArrow) {
        dir = ControlDirection::South;
    } else if input.key_pressed(NumCode::LeftArrow) {
        dir = ControlDirection::West;
    } else if input.key_pressed(NumCode::RightArrow) {
        dir = ControlDirection::East;
    } else {
        dir = ControlDirection::None;
    }

    ControlInfo { dir }
}
