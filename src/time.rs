use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::cell::Cell;

pub const TICK_BASE : u64 = 70;
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

pub fn new_ticker(time_count: TimeCount) -> Ticker {
    Ticker{time_count, last_count: Cell::new(0)}
}

impl Ticker {
    pub fn calc_tics(&self) -> u64 {
        
        let last_time_count = self.last_count.get();
        if last_time_count > get_count(&self.time_count) { // if the game was paused a LONG time
            set_count(&self.time_count, last_time_count);
        }

        let mut tics = 0;
        loop {
            let new_time = get_count(&self.time_count);
            tics = new_time - last_time_count;
            if tics != 0 {
                break;
            }
        }

        if tics > MAX_TICS {
            set_count(&self.time_count, get_count(&self.time_count) - (tics - MAX_TICS));
            tics = MAX_TICS
        }
        tics
    }
}