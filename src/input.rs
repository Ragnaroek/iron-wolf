use super::time::{TimeCount, get_count};
use vgaemu::input;

pub struct Input {
	time: TimeCount,
	pub input_monitoring: input::InputMonitoring
}

pub fn init(time: TimeCount, input_monitoring: input::InputMonitoring ) -> Input {
	Input{time, input_monitoring}
} 

impl Input {
	pub fn wait_user_input(&self, delay: u64) -> bool {
		let last_count = get_count(&self.time);
		self.input_monitoring.clear_keyboard();
		loop {
			if self.input_monitoring.any_key_pressed() {
				return true;
			}

			if get_count(&self.time) - last_count > delay {
				break;
			}
		}
		false
	}
}

