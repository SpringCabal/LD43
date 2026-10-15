use spring::ResultExt;
use spring::rules_params::los;

pub fn set_unit_number(unit_id: i32, name: &str, value: f32) {
    spring::set_unit_rules_param_f32(unit_id, name, value, los::PUBLIC)
        .log_err(&alloc::format!("{name} of unit {unit_id}"));
}

pub fn check<T>(result: spring::Result<T>, call: &str, unit: impl core::fmt::Debug) {
    if let Err(error) = result {
        spring::log_warning!("ttld-wasm", "{call} for unit {unit:?} failed: {error:?}");
    }
}
