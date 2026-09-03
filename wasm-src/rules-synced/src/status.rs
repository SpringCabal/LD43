use alloc::format;

use alloc::string::String;

use spring::generated::owned::rules_params::{RulesParamType, RulesParamValue};

use crate::state::World;

// LuaRulesParams::RULESPARAMLOS_PUBLIC.  A los mask of zero hides the param
// from every reader, including the game's own UI.
const LOS_PUBLIC: i32 = 32;

pub fn publish(world: &World, frame: i32) {
    let player = world.player.unit_id().unwrap_or(-1);
    let (health, max_health) = health(player);
    set_number("villagersKilled", world.villagers_killed as f32);
    set_number("alliesDrained", world.allies_drained as f32);
    set_number("orksKilled", world.orks.killed() as f32);
    set_string("gameEnd", world.game_end_state());
    let message = format!(
        "TTLD_STATUS|frame={frame}|health={health}|maxHealth={max_health}|villagersKilled={}|alliesDrained={}|orksKilled={}|gameEnd={}|playerAlive={}|casting={}|playerUnit={player}|castStart={}|castAnimTime={}",
        world.villagers_killed,
        world.allies_drained,
        world.orks.killed(),
        world.game_end_state(),
        i32::from(player >= 0),
        i32::from(world.player.casting(frame)),
        world.player.cast_start_frame(),
        world.player.cast_animation_time(),
    );
    let _ = spring::send_to_unsynced(&message);
}

fn health(unit_id: i32) -> (i32, i32) {
    if unit_id < 0 {
        return (0, 1000);
    }
    spring::units_info::get_unit_health(unit_id)
        .map(|health| (health.health.max(0.0) as i32, health.max_health as i32))
        .unwrap_or((0, 1000))
}

fn set_number(name: &str, value: f32) {
    let value = RulesParamValue {
        type_: RulesParamType::RulesparamTypeFloat,
        bool_value: false,
        float_value: value,
        string_value: String::new(),
    };
    let _ = spring::generated::owned::rules_params::set_game_rules_param(name, &value, LOS_PUBLIC);
}

fn set_string(name: &str, value: &str) {
    let value = RulesParamValue {
        type_: RulesParamType::RulesparamTypeString,
        bool_value: false,
        float_value: 0.0,
        string_value: String::from(value),
    };
    let _ = spring::generated::owned::rules_params::set_game_rules_param(name, &value, LOS_PUBLIC);
}
