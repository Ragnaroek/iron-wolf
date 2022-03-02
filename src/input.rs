use super::time::{TimeCount};
use std::sync::Arc;

pub struct Input {
	time: Arc<TimeCount>
}

pub fn init(time: Arc<TimeCount>) -> Input {
	Input{time}
} 

impl Input {
	pub fn user_input(&self, delay: u64) -> bool {
		let last_count = self.time.count();
		loop {
			// TODO Poll user input here
			if self.time.count() - last_count > delay {
				break;
			}
		}
		false
	}
}