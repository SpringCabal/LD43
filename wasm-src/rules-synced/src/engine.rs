//! Small wrappers over engine calls that several modules share.

use spring::ResultExt;
use spring::rules_params::los;

/// Publishes a unit number every reader can see.
pub fn set_unit_number(unit_id: i32, name: &str, value: f32) {
    spring::set_unit_rules_param_f32(unit_id, name, value, los::PUBLIC)
        .log_err(&alloc::format!("{name} of unit {unit_id}"));
}

/// Logs a failed engine call that gameplay depends on (an order, a unit
/// transfer or kill, damage), naming the call and the unit it was for.
pub fn check<T>(result: spring::Result<T>, call: &str, unit: impl core::fmt::Debug) {
    if let Err(error) = result {
        spring::log_warning!("ttld-wasm", "{call} for unit {unit:?} failed: {error:?}");
    }
}
