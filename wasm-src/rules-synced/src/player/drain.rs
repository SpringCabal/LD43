//! Blood magic: spells are paid for with the health of nearby allies.

use alloc::vec::Vec;

use spring::types::Float3;

use crate::defs::{UnitKind, defs};
use crate::fx::{ceg, play_sound_at, spawn_ceg, spawn_effect};
use crate::units::{self, NEUTRAL_TEAM, PLAYER_TEAM, def_of, health, health_value};

const BLOOD_MAGIC_RANGE: f32 = 600.0;
const LIFE_BONUS: f32 = 500.0;
const DRAIN_SEARCH_LIMIT: i32 = 50;
const CEG_SINGLE_BLOOD: &str = "feature_poof_single";
const CEG_FLAMES: &str = "feature_poof_spawner";
const SOUND_DRAIN: &str = "sounds/blooddrain.wav";

pub(super) fn drain_for_transfusion(unit_id: i32) -> Option<Option<f32>> {
    let health = health(unit_id)?;
    if health.current >= health.max {
        return None;
    }
    let required = LIFE_BONUS + health.max - health.current;
    let (found, shortfall) = drain(unit_id, required, true);
    if !found && shortfall.is_none() {
        return None;
    }
    Some(shortfall.map(|value| value - LIFE_BONUS))
}

pub(super) fn drain(collector_id: i32, required: f32, allow_partial: bool) -> (bool, Option<f32>) {
    let Some(origin) = units::position(collector_id) else {
        return (false, None);
    };
    let house = defs().id(UnitKind::House);
    let mut remaining = required;
    let mut to_kill: Vec<i32> = Vec::new();
    let mut to_restore: Vec<i32> = Vec::new();
    for _ in 0..DRAIN_SEARCH_LIMIT {
        let Ok(target) = spring::get_unit_nearest_ally(collector_id, BLOOD_MAGIC_RANGE) else {
            break;
        };
        let target = target.0;
        if target < 0 {
            break;
        }
        if def_of(target) == house {
            let _ = spring::transfer_unit(target, NEUTRAL_TEAM, false, false);
            to_restore.push(target);
            continue;
        }
        let Some(health) = health(target) else {
            break;
        };
        let available = health.current + LIFE_BONUS;
        if available > remaining {
            spawn_effect(target, CEG_FLAMES);
            play_sound_at(SOUND_DRAIN, origin);
            blood_trail(origin, target);
            let _ = spring::set_unit_health(target, health_value(available - remaining));
            restore(&to_restore);
            record_drained(consume(&to_kill, origin));
            return (true, None);
        }
        to_kill.push(target);
        let _ = spring::transfer_unit(target, NEUTRAL_TEAM, false, false);
        remaining -= available;
    }
    restore(&to_restore);
    if allow_partial && !to_kill.is_empty() {
        record_drained(consume(&to_kill, origin));
        return (false, Some(remaining));
    }
    restore(&to_kill);
    (false, None)
}

fn consume(units: &[i32], origin: Float3) -> i32 {
    let mut drained = 0;
    for unit_id in units {
        spawn_effect(*unit_id, CEG_FLAMES);
        play_sound_at(SOUND_DRAIN, origin);
        blood_trail(origin, *unit_id);
        let _ = spring::destroy_unit(
            *unit_id,
            spring::generated::owned::unit_control::DestroyUnitOptions {
                selfd: true,
                reclaimed: false,
                attacker_id: -1,
                recycle_id: false,
            },
        );
        drained += 1;
    }
    drained
}

fn record_drained(count: i32) {
    let _ = crate::state::with_world(|world| world.allies_drained += count);
}

fn restore(units: &[i32]) {
    for unit_id in units {
        let _ = spring::transfer_unit(*unit_id, PLAYER_TEAM, false, false);
    }
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
