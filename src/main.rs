extern crate iw;

#[cfg(feature = "tracing")]
use tracing;
#[cfg(feature = "tracing")]
use tracing_appender;
#[cfg(feature = "tracing")]
use tracing_subscriber;

use iw::assets;
use iw::config::read_iw_config;
use iw::loader::DiskLoader;
use iw::start::iw_start;

fn main() -> Result<(), String> {
    #[cfg(feature = "tracing")]
    let _guard = setup_tracing();

    let variant = &assets::W3D6; // TODO determine this with conditional compilation
    let iw_config = read_iw_config()?;
    let loader = DiskLoader {
        variant,
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path: iw_config.data.patch_data.clone(),
    };

    iw_start(loader, iw_config)
}

#[cfg(feature = "tracing")]
fn setup_tracing() -> tracing_appender::non_blocking::WorkerGuard {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file = format!("{}.trace", now);
    let appender = tracing_appender::rolling::never("./", file);
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(appender);

    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_writer(non_blocking_appender)
        .init();

    guard
}
