use vga::input::{self, InputMonitoring, MouseButton, NumCode};

use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    config::WolfConfig,
    def::{Button, NUM_BUTTONS, NUM_MOUSE_BUTTONS},
};

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
    pub button_0: bool,
    pub button_1: bool,
    pub button_2: bool,
    pub button_3: bool,
    pub dir: ControlDirection,
}

#[derive(PartialEq)]
pub enum InputMode {
    Player,
    DemoPlayback,
}

pub struct Input {
    time: TimeCount,
    pub input_monitoring: Arc<Mutex<input::InputMonitoring>>,
    pub demo_buffer: Option<Vec<u8>>,
    pub demo_ptr: usize,
    pub mode: InputMode,
    pub mouse_enabled: bool,
    pub joystick_enabled: bool,
    pub button_scan: [NumCode; NUM_BUTTONS],
    pub button_mouse: [Button; NUM_MOUSE_BUTTONS],
    pub dir_scan: [NumCode; 4],
}

// Indexes into the Input.dir_scan array for the up, down, left, right buttons
pub const DIR_SCAN_NORTH: usize = 0;
pub const DIR_SCAN_EAST: usize = 1;
pub const DIR_SCAN_SOUTH: usize = 2;
pub const DIR_SCAN_WEST: usize = 3;

impl Input {
    pub fn init_player(
        time: TimeCount,
        input_monitoring: Arc<Mutex<input::InputMonitoring>>,
        wolf_config: &WolfConfig,
    ) -> Input {
        Input {
            time,
            mouse_enabled: true,
            joystick_enabled: false,
            input_monitoring,
            demo_buffer: None,
            demo_ptr: 0,
            mode: InputMode::Player,
            button_scan: wolf_config.button_scan.clone(),
            button_mouse: wolf_config.button_mouse.clone(),
            dir_scan: wolf_config.dir_scan.clone(),
        }
    }

    pub fn init_demo_playback(
        time: TimeCount,
        input_monitoring: Arc<Mutex<input::InputMonitoring>>,
        demo_buffer: Vec<u8>,
    ) -> Input {
        Input {
            time,
            mouse_enabled: false,
            joystick_enabled: false,
            input_monitoring,
            demo_buffer: Some(demo_buffer),
            demo_ptr: 0,
            mode: InputMode::DemoPlayback,
            button_scan: [NumCode::None; NUM_BUTTONS],
            button_mouse: [Button::NoButton; NUM_MOUSE_BUTTONS],
            dir_scan: [NumCode::None; 4],
        }
    }

    pub fn wait_user_input(&self, delay: u64) -> bool {
        let last_count = get_count(&self.time);
        {
            let mut input = self.im_lock();
            input.clear_keyboard();
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

    pub fn ack(&self) -> bool {
        self.wait_user_input(u64::MAX)
    }

    pub fn start_ack(&self) {
        let mut input = self.im_lock();
        input.clear_keyboard();
        input.clear_mouse();
        // TODO clear joystick buttons
    }

    pub fn check_ack(&self) -> bool {
        let input = self.im_lock();
        input.any_key_pressed()
    }

    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        let input = self.im_lock();
        input.mouse_button_pressed(button)
    }

    pub fn key_pressed(&self, code: NumCode) -> bool {
        let input = self.im_lock();
        input.key_pressed(code)
    }

    pub fn clear_keys_down(&self) {
        if self.mode == InputMode::Player {
            let mut input = self.im_lock();
            input.clear_keyboard();
            input.keyboard.last_scan = NumCode::None;
            input.keyboard.last_ascii = '\0';
        }
    }

    pub fn clear_last_scan(&self) {
        let mut input = self.im_lock();
        input.keyboard.last_scan = NumCode::None;
    }

    pub fn last_scan(&self) -> NumCode {
        let input = self.im_lock();
        input.keyboard.last_scan
    }

    pub fn clear_last_ascii(&self) {
        let mut input = self.im_lock();
        input.keyboard.last_ascii = '\0';
    }

    // Returns the 0 char if nothing is set
    pub fn last_ascii(&self) -> char {
        let input = self.im_lock();
        input.keyboard.last_ascii
    }

    fn im_lock(&self) -> MutexGuard<'_, InputMonitoring> {
        self.input_monitoring.lock().expect("input monitor lock")
    }
}

//	read_control() - Reads the device associated with the specified
//	player and fills in the control info struct
pub fn read_control(input: &Input, ci: &mut ControlInfo) {
    if input.key_pressed(NumCode::UpArrow) {
        ci.dir = ControlDirection::North;
    } else if input.key_pressed(NumCode::DownArrow) {
        ci.dir = ControlDirection::South;
    } else if input.key_pressed(NumCode::LeftArrow) {
        ci.dir = ControlDirection::West;
    } else if input.key_pressed(NumCode::RightArrow) {
        ci.dir = ControlDirection::East;
    } else {
        ci.dir = ControlDirection::None;
    }
}
