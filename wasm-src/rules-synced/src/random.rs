//! The synced random numbers the rules and unit scripts draw, from the engine
//! RNG that the Lua build's `math.random` uses.

use spring::random::synced;

/// A float in `[0, 1)`, like `math.random()`.
pub fn unit_float() -> f32 {
    synced::float()
}

/// A float in `[low, high)`.
pub fn range(low: f32, high: f32) -> f32 {
    synced::range(low, high)
}

/// A float within `spread / 2` of zero, like `math.random() * spread - spread / 2`.
pub fn centered(spread: f32) -> f32 {
    spread * unit_float() - spread / 2.0
}

/// A random building facing, like `math.floor(math.random() * 4)`.
pub fn facing() -> i32 {
    (unit_float() * 4.0) as i32
}
