//! What the mouse cursor points at.

/// The ground under a screen position (through any unit on it), and the unit
/// under it. A trace that misses the map is `None`, never a position.
pub use springcabal_control::pointer::{ground_at, unit_at};

/// The cursor's screen position, or `None` while it is off the window.
pub fn screen() -> Option<(f32, f32)> {
    springcabal_control::pointer::mouse().map(|mouse| (mouse.x, mouse.y))
}
