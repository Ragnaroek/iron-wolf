use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub const TICK_BASE : u64 = 70;

pub type TimeCount = Arc<AtomicU64>;

pub fn new() -> TimeCount {
    Arc::new(AtomicU64::new(0))
}

pub fn count(count: &TimeCount) -> u64 {
    count.load(Ordering::Relaxed)
}