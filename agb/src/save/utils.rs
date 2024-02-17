//! A package containing useful utilities for writing save accessors.

use super::Error;
use crate::{
    sync::{RawLock, RawLockGuard},
    timer::{Divider, Timer},
};

/// A timeout type used to prevent hardware errors in save media from hanging
/// the game.
pub struct Timeout {
    timer: Option<Timer>,
}
impl Timeout {
    /// Creates a new timeout from the timer passed to [`set_timer_for_timeout`].
    ///
    /// ## Errors
    ///
    /// If another timeout has already been created.
    #[inline(never)]
    pub fn new(timer: Option<Timer>) -> Self {
        Timeout { timer }
    }

    /// Starts this timeout.
    pub fn start(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.set_enabled(false);
            timer.set_divider(Divider::Divider1024);
            timer.set_interrupt(false);
            timer.set_overflow_amount(0xFFFF);
            timer.set_cascade(false);
            timer.set_enabled(true);
        }
    }

    /// Returns whether a number of milliseconds has passed since the last call
    /// to [`Timeout::start()`].
    pub fn check_timeout_met(&self, check_ms: u16) -> bool {
        if let Some(timer) = &self.timer {
            check_ms * 17 < timer.value()
        } else {
            false
        }
    }
}
impl Drop for Timeout {
    fn drop(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.set_enabled(false);
        }
    }
}

pub fn lock_media_access() -> Result<RawLockGuard<'static>, Error> {
    static LOCK: RawLock = RawLock::new();
    match LOCK.try_lock() {
        Some(x) => Ok(x),
        None => Err(Error::MediaInUse),
    }
}
