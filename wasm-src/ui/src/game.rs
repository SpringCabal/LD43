//! The game clock as the UI sees it.

use ttld_shared::STATUS_PERIOD;

/// The frame of the last status check: the current frame rounded down to the
/// status period.
pub fn status_frame() -> i32 {
    let frame = spring::game::current_frame();
    frame - frame % STATUS_PERIOD
}
