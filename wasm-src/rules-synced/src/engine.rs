//! Small wrappers over engine calls that several modules share.

use alloc::string::String;

use spring::generated::owned::rules_params::{self, RulesParamType, RulesParamValue};
use spring::rules_params::los;

/// The current game frame.
pub fn frame() -> i32 {
    spring::game::get_game_frame(0)
        .map(|frame| (frame.low16 | (frame.high16 << 16)) as i32)
        .unwrap_or_default()
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
