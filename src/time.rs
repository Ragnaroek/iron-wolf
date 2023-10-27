use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::cell::Cell;
use std::thread;

pub const TICK_BASE : u64 = 70; //Hz
const TARGET_NANOS : u128 = 1_000_000_000 / TICK_BASE as u128;
const MAX_TICS: u64 = 10;

pub type TimeCount = Arc<AtomicU64>;

pub fn new_time_count() -> TimeCount {
    Arc::new(AtomicU64::new(0))
}

pub fn get_count(count: &TimeCount) -> u64 {
    count.load(Ordering::Relaxed)
}

pub fn set_count(count: &TimeCount, new_val: u64) {
    count.store(new_val, Ordering::Relaxed)
}

pub struct Ticker {
    pub time_count: TimeCount,
    pub last_count: Cell<u64>,
}

pub fn new_ticker() -> Ticker {
    let time_count = new_time_count();
    let time_t = time_count.clone();
    thread::spawn(move || { 
		let mut last_tick = std::time::Instant::now();
		loop {
			let last_duration = last_tick.elapsed().as_nanos();
			let overlap = (last_duration as i128 - TARGET_NANOS as i128).clamp(0, TARGET_NANOS as i128);    
		
			last_tick = std::time::Instant::now();
			thread::sleep(std::time::Duration::from_nanos((TARGET_NANOS - overlap as u128) as u64));
			time_t.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
		}
    });

    Ticker{time_count, last_count: Cell::new(0)}
}

impl Ticker {

    pub fn get_count(&self) -> u64 {
        get_count(&self.time_count)
    }

    pub fn calc_tics(&self) -> u64 {
        let last_time_count = self.last_count.get();
        if last_time_count > get_count(&self.time_count) { // if the game was paused a LONG time
            set_count(&self.time_count, last_time_count);
        }

        let mut tics;
        let mut new_time;
        loop {
            new_time = get_count(&self.time_count);
            tics = new_time - last_time_count;
            if tics != 0 {
                break;
            }
        }
        self.last_count.set(new_time);

        if tics > MAX_TICS {
            set_count(&self.time_count, get_count(&self.time_count) - (tics - MAX_TICS));
            tics = MAX_TICS
        }
        tics
    }
}