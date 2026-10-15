use alloc::vec::Vec;

use spring::commands;
use spring::types::Float3;
use spring::units_info;

use crate::defs::{UnitKind, defs};
use crate::fx::{ceg, play_sound, spawn_ceg, spawn_effect};
use crate::state::Global;
use crate::units::{self, NEUTRAL_TEAM, PLAYER_TEAM, def_of, health, health_value};

const BLOOD_MAGIC_RANGE: f32 = 600.0;
const LIFE_BONUS: f32 = 500.0;
const DRAIN_SEARCH_LIMIT: usize = 50;
const MIDDLE_MARGIN: f32 = 100.0;
const ALL_UNITS: i32 = -1;
const CEG_SINGLE_BLOOD: &str = "feature_poof_single";
const CEG_FLAMES: &str = "feature_poof_spawner";
const SOUND_DRAIN: &str = "sounds/blooddrain.wav";
const SOUND_DRAIN_VOLUME: f32 = 30.0;

pub(super) fn drain_for_transfusion(global: &Global, unit_id: i32) -> Option<Option<f32>> {
    let health = health(unit_id)?;
    if health.current >= health.max {
        return None;
    }
    let required = LIFE_BONUS + health.max - health.current;
    let (found, shortfall) = drain(global, unit_id, required, true);
    if !found && shortfall.is_none() {
        return None;
    }
    Some(shortfall.map(|value| value - LIFE_BONUS))
}

pub(super) fn drain(
    global: &Global,
    collector_id: i32,
    required: f32,
    allow_partial: bool,
) -> (bool, Option<f32>) {
    let Some(origin) = units::position(collector_id) else {
        return (false, None);
    };
    let house = defs().id(UnitKind::House);
    let mut remaining = required;
    let mut to_kill: Vec<i32> = Vec::new();
    let mut to_restore: Vec<i32> = Vec::new();
    for target in nearest_allies(collector_id, origin)
        .into_iter()
        .take(DRAIN_SEARCH_LIMIT)
    {
        if def_of(target) == house {
            transfer(target, NEUTRAL_TEAM);
            to_restore.push(target);
            continue;
        }
        let Some(health) = health(target) else {
            break;
        };
        let available = health.current + LIFE_BONUS;
        if available > remaining {
            drain_effect(target);
            blood_trail(origin, target);
            crate::engine::check(
                spring::set_unit_health(target, health_value(available - remaining)),
                "set_unit_health",
                target,
            );
            restore(&to_restore);
            record_drained(global, consume(&to_kill, origin));
            return (true, None);
        }
        to_kill.push(target);
        transfer(target, NEUTRAL_TEAM);
        remaining -= available;
    }
    restore(&to_restore);
    if allow_partial && !to_kill.is_empty() {
        record_drained(global, consume(&to_kill, origin));
        return (false, Some(remaining));
    }
    restore(&to_kill);
    (false, None)
}

fn nearest_allies(collector_id: i32, origin: Float3) -> Vec<i32> {
    let Ok(ally_team) = units_info::get_unit_ally_team(collector_id) else {
        return Vec::new();
    };
    let range_squared = BLOOD_MAGIC_RANGE * BLOOD_MAGIC_RANGE;
    let mut allies: Vec<(f32, i32)> = units::units_in_cylinder(
        origin.x,
        origin.z,
        BLOOD_MAGIC_RANGE + MIDDLE_MARGIN,
        ALL_UNITS,
    )
    .into_iter()
    .filter(|unit_id| {
        *unit_id != collector_id && units_info::get_unit_ally_team(*unit_id).ok() == Some(ally_team)
    })
    .filter_map(|unit_id| {
        let middle = units_info::get_unit_position(
            unit_id,
            units_info::GetUnitPositionOptions {
                mid_pos: true,
                aim_pos: false,
            },
        )
        .ok()?;
        let dx = origin.x - middle.x;
        let dz = origin.z - middle.z;
        let distance_squared = dx * dx + dz * dz;
        (distance_squared <= range_squared).then_some((distance_squared, unit_id))
    })
    .collect();
    allies.sort_by(|a, b| a.0.total_cmp(&b.0));
    allies.into_iter().map(|(_, unit_id)| unit_id).collect()
}

fn consume(units: &[i32], origin: Float3) -> i32 {
    let mut drained = 0;
    for unit_id in units {
        let unit_id = *unit_id;
        drain_effect(unit_id);
        blood_trail(origin, unit_id);
        let options = spring::generated::owned::unit_control::DestroyUnitOptions {
            selfd: true,
            reclaimed: false,
            attacker_id: -1,
            recycle_id: false,
        };
        commands::destroy_unit(unit_id, options)
            .then(move |result| crate::engine::check(result, "destroy_unit", unit_id));
        drained += 1;
    }
    drained
}

fn drain_effect(unit_id: i32) {
    spawn_effect(unit_id, CEG_FLAMES);
    play_sound(SOUND_DRAIN, SOUND_DRAIN_VOLUME, unit_id);
}

fn record_drained(global: &Global, count: i32) {
    global.score.access_mut().allies_drained += count;
}

fn restore(units: &[i32]) {
    for unit_id in units {
        transfer(*unit_id, PLAYER_TEAM);
    }
}

fn transfer(unit_id: i32, team: i32) {
    commands::transfer_unit(unit_id, team, false, false)
        .then(move |result| crate::engine::check(result, "transfer_unit", unit_id));
}

fn blood_trail(origin: Float3, target_id: i32) {
    let Some(target) = units::position(target_id) else {
        return;
    };
    let dx = origin.x - target.x;
    let dy = origin.y - target.y;
    let dz = origin.z - target.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
    let step = Float3 {
        x: dx / distance,
        y: dy / distance,
        z: dz / distance,
    };
    let blood = ceg(CEG_SINGLE_BLOOD);
    let mut travelled = 10.0;
    while travelled < distance {
        spawn_ceg(
            Float3 {
                x: target.x + step.x * travelled,
                y: target.y + step.y * travelled,
                z: target.z + step.z * travelled,
            },
            &blood,
        );
        travelled += 10.0;
    }
}
