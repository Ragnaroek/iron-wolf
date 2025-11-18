#[cfg(feature = "tracing")]
use tracing::instrument;

use web_time::{Duration, Instant};

use vga::util::sleep;

pub const TICK_BASE: u64 = 70; //Hz
const TARGET_NANOS: u128 = 1_000_000_000 / TICK_BASE as u128; //duration of one tick in nanos
const TARGET_MILLIS: f64 = 1000.0 / TICK_BASE as f64;
// target frame duration at 70Hz
pub const TARGET_FRAME_DURATION: Duration = Duration::from_nanos(TARGET_NANOS as u64);

const MAX_TICS: u64 = 10;

pub struct Ticker {
    pub last_count: u64,
    pub ref_time: Instant,
}

pub fn new_ticker() -> Ticker {
    Ticker {
        last_count: 0,
        ref_time: Instant::now(),
    }
}

impl Ticker {
    pub fn get_count(&self) -> u64 {
        let elapsed = self.ref_time.elapsed().as_millis_f64();
        (elapsed / TARGET_MILLIS) as u64
    }

    // returns the count the next tic is based on
    pub fn next_tics_time(&self, delta_tic: u64) -> (Instant, u64) {
        let count = self.get_count();
        (
            self.ref_time
                + Duration::from_nanos(
                    ((count + delta_tic) as f64 * TARGET_MILLIS * 1_000_000.0) as u64,
                ),
            count,
        )
    }

    pub fn clear_count(&mut self) {
        self.ref_time = Instant::now();
        self.last_count = 0;
    }

    #[cfg_attr(feature = "tracing", instrument(skip_all))]
    pub async fn wait_for_tic(&mut self) -> u64 {
        if self.last_count > self.get_count() {
            // if the game was paused a LONG time
            self.clear_count();
        }

        let mut tics;
        let mut new_time;
        let mut get_times = Duration::ZERO;
        loop {
            let get_start = Instant::now();
            new_time = self.get_count();
            get_times += Instant::now() - get_start;
            tics = new_time.saturating_sub(self.last_count);
            if tics != 0 {
                break;
            }
        }
        self.last_count = new_time;

        if tics > MAX_TICS {
            self.clear_count();
            tics = MAX_TICS
        }
        tics
    }

    // waits for 'count' tics in a non-busy way
    pub async fn tics(&self, count: u64) {
        //sleep(Duration::from_nanos((TARGET_NANOS * count as u128) as u64)).await
        sleep(Duration::from_nanos((TARGET_NANOS * count as u128) as u64).as_millis_f64() as u32)
            .await
    }
}
