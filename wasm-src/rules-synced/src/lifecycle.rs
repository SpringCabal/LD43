//! Roster bookkeeping for created and destroyed units.

use crate::units::PLAYER_TEAM;
use crate::{combat, state, terrain};

fn custom_scale(def_id: i32, key: &str) -> Option<f32> {
    spring::get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|value: &f32| *value != 0.0)
}

fn publish_draw_scale(unit_id: i32, unit_def_id: i32) {
    let horizontal = custom_scale(unit_def_id, "hscale").unwrap_or(1.0);
    let vertical = custom_scale(unit_def_id, "vscale").unwrap_or(1.0);
    if horizontal == 1.0 && vertical == 1.0 {
        return;
    }
    let _ = spring::set_unit_rules_param_f32(
        unit_id,
        "scale_vertical",
        vertical,
        spring::rules_params::los::INLOS,
    );
    let _ = spring::set_unit_rules_param_f32(
        unit_id,
        "scale_horizontal",
        horizontal,
        spring::rules_params::los::INLOS,
    );
}

pub fn unit_created(unit_id: i32, unit_def_id: i32, unit_team: i32, _builder_id: i32) {
    crate::unit_scripts::queue_unit_created(unit_id, unit_def_id);
    publish_draw_scale(unit_id, unit_def_id);
    terrain::block(unit_id, unit_def_id);
    let _ = state::with_world(|world| world.player.adopt(unit_id, unit_def_id));
    if unit_team == PLAYER_TEAM {
        combat::add_effect(
            unit_id,
            combat::Effect::HalfSpeed,
            combat::Modifier {
                move_factor: 0.5,
                reload_factor: 1.0,
            },
        );
    }
    let registered = state::with_world(|world| world.orks.created(unit_id, unit_def_id, unit_team))
        .unwrap_or(false);
    if registered {
        combat::advance(unit_id);
    }
}

pub fn unit_destroyed(
    unit_id: i32,
    unit_def_id: i32,
    _unit_team: i32,
    _attacker_id: i32,
    _attacker_def_id: i32,
    _attacker_team: i32,
    _weapon_def_id: i32,
) {
    terrain::unblock(unit_id, unit_def_id);
    combat::forget(unit_id);
    combat::status_forget(unit_id);
    let _ = state::with_world(|world| {
        world.remove_villager(unit_id);
        world.orks.destroyed(unit_id);
        world.player.release(unit_id);
    });
}
