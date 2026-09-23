//! The game clock as the UI sees it.

/// How often, in frames, the rules check the status and the fire effect
/// checks for burning houses.
const STATUS_PERIOD: i32 = 6;

/// The current game frame.
pub fn frame() -> i32 {
    spring::game::get_game_frame(0)
        .map(|frame| (frame.low16 | (frame.high16 << 16)) as i32)
        .unwrap_or_default()
}

/// The frame of the last status check: the current frame rounded down to the
/// status period.
pub fn status_frame() -> i32 {
    let frame = frame();
    frame - frame % STATUS_PERIOD
}
