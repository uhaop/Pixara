use std::sync::atomic::{AtomicBool, Ordering};

static CANCEL_BATCH: AtomicBool = AtomicBool::new(false);

pub fn reset_batch_cancel() {
    CANCEL_BATCH.store(false, Ordering::SeqCst);
}

pub fn request_batch_cancel() {
    CANCEL_BATCH.store(true, Ordering::SeqCst);
}

pub fn is_batch_cancelled() -> bool {
    CANCEL_BATCH.load(Ordering::SeqCst)
}
