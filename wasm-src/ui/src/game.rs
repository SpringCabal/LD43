//! The game clock as the UI sees it.

/// How often, in frames, the rules check the status and the fire effect
/// checks for burning houses.
const STATUS_PERIOD: i32 = 6;

/// The frame of the last status check: the current frame rounded down to the
/// status period.
pub fn status_frame() -> i32 {
    let frame = spring::game::current_frame();
    frame - frame % STATUS_PERIOD
}
