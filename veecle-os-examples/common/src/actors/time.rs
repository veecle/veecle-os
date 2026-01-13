//! A minimal example using the time abstraction.

use core::fmt::Debug;

use veecle_os::osal::api::time::{Duration, Instant, Interval, TimeAbstraction};
use veecle_os::runtime::{InitializedReader, Never, Storable, Writer};
use veecle_os::telemetry::{error, info};

const INTERVAL_PERIOD: Duration = Duration::from_secs(1);
const ABS_ERROR: Duration = Duration::from_millis(10);

/// A clock tick, which happens in an specific moment in time.
#[derive(Debug, PartialEq, Clone, Storable)]
pub struct Tick {
    at: Instant,
}

/// Emits [`Tick`] every second.
#[veecle_os::runtime::actor]
pub async fn ticker_actor<T>(
    mut tick_writer: Writer<'_, Tick>,
) -> Result<Never, veecle_os::osal::api::Error>
where
    T: TimeAbstraction,
{
    let mut interval = T::interval(INTERVAL_PERIOD);

    loop {
        interval.tick().await?;
        tick_writer.write(Tick { at: T::now() }).await;
    }
}

/// Prints every received tick via `telemetry` (if enabled).
/// Additionally, prints to stderr if the time between
/// the previous and current tick differs by more than 10 millis.
#[veecle_os::runtime::actor]
pub async fn ticker_reader(mut reader: InitializedReader<'_, Tick>) -> Never {
    let mut previous: Option<Instant> = None;

    loop {
        reader
            .wait_for_update()
            .await
            .read(|&Tick { at: tick_at }| {
                info!("last tick was at", tick_at = format_args!("{tick_at:?}"));

                if let Some(previous) = previous
                    && let Some(elapsed) = tick_at.duration_since(previous)
                {
                    info!("since last tick", elapsed = format_args!("{elapsed:?}"));
                }

                if previous
                    .replace(tick_at)
                    .and_then(|previous| tick_at.duration_since(previous))
                    .map(|diff| diff.abs_diff(INTERVAL_PERIOD) > ABS_ERROR)
                    .unwrap_or_default()
                {
                    error!(
                        "previous and latest tick differ more than",
                        period = i64::try_from(INTERVAL_PERIOD.as_millis()).unwrap(),
                        error_bound = i64::try_from(ABS_ERROR.as_millis()).unwrap()
                    );
                }
            });
    }
}
