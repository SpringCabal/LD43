pub use spring::pick::{ground_at, unit_at};

pub fn screen() -> Option<(f32, f32)> {
    springcabal_control::pointer::mouse().map(|mouse| (mouse.x, mouse.y))
}
