//! Ctrl-C handler. Sets a shared `AtomicBool`; stages check it between heavy steps.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static TRIGGERED: AtomicBool = AtomicBool::new(false);

pub fn install() {
    let _ = ctrlc::set_handler(|| {
        TRIGGERED.store(true, Ordering::SeqCst);
        // Keep this short — handlers run on signal thread.
    });
}

pub fn triggered() -> bool {
    TRIGGERED.load(Ordering::SeqCst)
}

#[allow(dead_code)]
pub fn as_flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}