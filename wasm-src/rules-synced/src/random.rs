use core::cell::Cell;

const SEED: u64 = 0x2545_f491_4f6c_dd1d;
const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const INCREMENT: u64 = 1_442_695_040_888_963_407;

thread_local! {
    static SEQUENCE: Cell<u64> = const { Cell::new(SEED) };
}

pub fn unit_float() -> f32 {
    let bits = (next() >> 40) as u32;
    bits as f32 / 16_777_216.0
}

pub fn range(low: f32, high: f32) -> f32 {
    low + (high - low) * unit_float()
}

pub fn centered(spread: f32) -> f32 {
    spread * unit_float() - spread / 2.0
}

pub fn facing() -> i32 {
    (unit_float() * 4.0) as i32
}

fn next() -> u64 {
    SEQUENCE.with(|sequence| {
        let value = sequence
            .get()
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(INCREMENT);
        sequence.set(value);
        value
    })
}
