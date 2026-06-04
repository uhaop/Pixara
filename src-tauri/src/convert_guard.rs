use std::sync::atomic::{AtomicBool, Ordering};

static CONVERT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub fn is_convert_in_progress() -> bool {
    CONVERT_IN_PROGRESS.load(Ordering::SeqCst)
}

/// Ensures the global convert lock is released even if the batch task panics.
pub struct ConvertInProgressGuard;

impl ConvertInProgressGuard {
    pub fn try_acquire() -> Result<Self, &'static str> {
        if CONVERT_IN_PROGRESS
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err("A conversion batch is already running");
        }
        Ok(Self)
    }
}

impl Drop for ConvertInProgressGuard {
    fn drop(&mut self) {
        CONVERT_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}
