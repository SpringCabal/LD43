use spring::random::synced;

pub fn unit_float() -> f32 {
    synced::float()
}

pub fn range(low: f32, high: f32) -> f32 {
    synced::range(low, high)
}

pub fn centered(spread: f32) -> f32 {
    spread * unit_float() - spread / 2.0
}

pub fn facing() -> i32 {
    (unit_float() * 4.0) as i32
}
