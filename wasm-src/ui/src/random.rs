//! The deterministic LCG behind the UI's decorative randomness (house lights,
//! fire particles).

pub fn next(seed: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

/// A float in `[0, 1)` from the high bits of `seed`.
pub fn unit_float(seed: u64) -> f32 {
    ((seed >> 40) as u32) as f32 / 16_777_216.0
}
