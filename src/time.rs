#[cfg(feature = "tracing")]
use tracing::instrument;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_std::task;

pub const TICK_BASE: u64 = 70; //Hz
const TARGET_NANOS: u128 = 1_000_000_000 / TICK_BASE as u128; //duration of one tick in nanos
const TARGET_MILLIS: f64 = 1000.0 / TICK_BASE as f64;
const TICK_SAMPLE_RATE: Duration = std::time::Duration::from_nanos((TARGET_NANOS / 2) as u64);

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
    pub last_count: AtomicU64,
}

pub fn new_ticker() -> Ticker {
    let time_count = new_time_count();
    let time_t = time_count.clone();

    task::spawn_blocking(move || {
        let start_time = Instant::now();
        loop {
            std::thread::sleep(TICK_SAMPLE_RATE);
            let elapsed = start_time.elapsed().as_millis_f64();
            let tics = (elapsed / TARGET_MILLIS) as u64;
            time_t.store(tics, std::sync::atomic::Ordering::Relaxed);
        }
    });

    Ticker {
        time_count,
        last_count: AtomicU64::new(0),
    }
}

impl Ticker {
    pub fn get_count(&self) -> u64 {
        get_count(&self.time_count)
    }

    pub fn clear_count(&self) {
        set_count(&self.time_count, 0)
    }

    #[cfg_attr(feature = "tracing", instrument(skip_all))]
    pub async fn wait_for_tic(&self) -> u64 {
        let last_time_count = self.last_count.load(Ordering::Relaxed);
        if last_time_count > get_count(&self.time_count) {
            // if the game was paused a LONG time
            set_count(&self.time_count, last_time_count);
        }

        let mut tics;
        let mut new_time;
        let mut get_times = Duration::ZERO;
        loop {
            let get_start = Instant::now();
            new_time = get_count(&self.time_count);
            get_times += Instant::now() - get_start;
            tics = new_time - last_time_count;
            if tics != 0 {
                break;
            }
        }
        self.last_count.store(new_time, Ordering::Relaxed);

        if tics > MAX_TICS {
            set_count(
                &self.time_count,
                get_count(&self.time_count) - (tics - MAX_TICS),
            );
            tics = MAX_TICS
        }
        tics
    }

    // waits for 'count' tics in a non-busy way
    pub async fn tics(&self, count: u64) {
        task::sleep(std::time::Duration::from_nanos(
            (TARGET_NANOS * count as u128) as u64,
        ))
        .await
    }
}
