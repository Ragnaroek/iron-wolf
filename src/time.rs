use std::thread;
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc};

pub const TICK_BASE : u64 = 70;

pub struct TimeCount {
	counter: Arc<AtomicU64>
}

pub fn init() -> TimeCount {
	let counter = AtomicU64::new(0);
	let counter_t = Arc::new(counter);
	let counter_r = counter_t.clone();

	thread::spawn(move || { 
		let mut last_tick = Instant::now();
		loop {
			let last_duration = last_tick.elapsed().as_millis();
			let overlap = if last_duration > 14 {
				(last_duration as u64 - 14).min(0).max(14)
			} else {
				0
			};
			last_tick = Instant::now();
			thread::sleep(Duration::from_millis(14 - overlap));
			counter_t.fetch_add(1, Ordering::Relaxed);
		}
    });

	TimeCount{counter: counter_r}
}

impl TimeCount {
	pub fn count(&self) -> u64 {
		self.counter.load(Ordering::Relaxed)
	}
}