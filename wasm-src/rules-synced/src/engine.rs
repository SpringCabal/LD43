//! Small wrappers over engine calls that several modules share.

use alloc::string::String;

use spring::generated::owned::rules_params::{self, RulesParamType, RulesParamValue};
use spring::rules_params::los;

/// The current game frame.
pub fn frame() -> i32 {
    spring::game::frame().unwrap_or_default() as i32
}

/// Publishes a number every reader can see, the game's own UI included.
pub fn set_game_number(name: &str, value: f32) {
    let value = RulesParamValue {
        type_: RulesParamType::RulesparamTypeFloat,
        bool_value: false,
        float_value: value,
        string_value: String::new(),
    };
    let _ = rules_params::set_game_rules_param(name, &value, los::PUBLIC);
}

/// Publishes a unit number every reader can see.
pub fn set_unit_number(unit_id: i32, name: &str, value: f32) {
    let _ = spring::set_unit_rules_param_f32(unit_id, name, value, los::PUBLIC);
}

/// Logs a failed engine call that gameplay depends on (an order, a unit
/// transfer or kill, damage), naming the call and the unit it was for.
pub fn check<T>(result: spring::Result<T>, call: &str, unit: impl core::fmt::Debug) {
    if let Err(error) = result {
        spring::log_warning!("ttld-wasm", "{call} for unit {unit:?} failed: {error:?}");
    }
}
