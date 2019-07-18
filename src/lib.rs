pub use mio_old::{Event, PollOpt, Ready, Token};

mod evented;
mod events;
mod poll;
mod registration;
pub mod event {
    pub use crate::evented::Evented;
    pub use crate::events::Events;
    pub use mio_old::Event;
}
pub mod net;

pub use event::Events;
pub use evented::Evented;
pub use poll::Poll;
pub use registration::{Registration, SetReadiness};

#[cfg(feature = "with-deprecated")]
mod convert {
    use std::time::Duration;

    const NANOS_PER_MILLI: u32 = 1_000_000;
    const MILLIS_PER_SEC: u64 = 1_000;

    /// Convert a `Duration` to milliseconds, rounding up and saturating at
    /// `u64::MAX`.
    ///
    /// The saturating is fine because `u64::MAX` milliseconds are still many
    /// million years.
    pub fn millis(duration: Duration) -> u64 {
        // Round up.
        let millis = (duration.subsec_nanos() + NANOS_PER_MILLI - 1) / NANOS_PER_MILLI;
        duration
            .as_secs()
            .saturating_mul(MILLIS_PER_SEC)
            .saturating_add(u64::from(millis))
    }
}
