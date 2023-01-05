use super::time::{TimeCount};
use std::sync::Arc;
use vgaemu::input;

pub struct Input {
	time: Arc<TimeCount>,
	pub input_monitoring: input::InputMonitoring
}

pub fn init(time: Arc<TimeCount>, input_monitoring: input::InputMonitoring ) -> Input {
	Input{time, input_monitoring}
} 

impl Input {
	pub fn wait_user_input(&self, delay: u64) -> bool {
		let last_count = self.time.count();
		self.input_monitoring.clear_keyboard();
		loop {
			if self.input_monitoring.any_key_pressed() {
				return true;
			}

			if self.time.count() - last_count > delay {
				break;
			}
		}
		false
	}
}

