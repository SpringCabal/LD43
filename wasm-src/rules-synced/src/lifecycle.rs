//! Roster bookkeeping for created and destroyed units.

use crate::units::PLAYER_TEAM;
use crate::{combat, state, terrain};

pub fn unit_created(unit_id: i32, unit_def_id: i32, unit_team: i32, _builder_id: i32) {
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
