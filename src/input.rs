use vga::input::{self, NumCode};

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
	pub input_monitoring: input::InputMonitoring
}

pub fn init(time: TimeCount, input_monitoring: input::InputMonitoring ) -> Input {
	Input{time, input_monitoring}
} 

impl Input {
	pub async fn wait_user_input(&self, delay: u64) -> bool {
		let last_count = get_count(&self.time);
		self.input_monitoring.clear_keyboard();
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
        self.input_monitoring.clear_keyboard();
        // TODO clear mouse and joystick buttons
    }

    pub fn check_ack(&self) -> bool {
        self.input_monitoring.any_key_pressed()
    }

    pub fn key_pressed(&self, code: NumCode) -> bool {
        self.input_monitoring.key_pressed(code)
    }

	pub fn clear_keys_down(&self) {
		// TODO set LastScan to None
		// TODO set LastASCII to None
		self.input_monitoring.clear_keyboard();
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

    ControlInfo{
        dir
    }
}
