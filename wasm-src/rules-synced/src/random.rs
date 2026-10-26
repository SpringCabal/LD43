use spring::{RulesSynced, Spring};

#[cfg_attr(feature = "diagnostics", track_caller)]
pub fn unit_float(spring: &Spring<'_, RulesSynced>) -> f32 {
    let value = spring
        .synced_random()
        .next_float()
        .expect("synced random float");
    #[cfg(feature = "diagnostics")]
    if spring.game().current_frame().unwrap_or(2) <= 1 {
        spring::log_notice!(
            spring,
            "PerfRng",
            "{value:.9} {}",
            core::panic::Location::caller()
        );
    }
    value
}
pub fn range(spring: &Spring<'_, RulesSynced>, low: f32, high: f32) -> f32 {
    low + (high - low) * unit_float(spring)
}
pub fn centered(spring: &Spring<'_, RulesSynced>, spread: f32) -> f32 {
    spread * unit_float(spring) - spread / 2.0
}
pub fn facing(spring: &Spring<'_, RulesSynced>) -> i32 {
    (unit_float(spring) * 4.0) as i32
}
