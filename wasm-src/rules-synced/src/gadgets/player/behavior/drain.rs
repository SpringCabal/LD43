use alloc::vec::Vec;

use spring::Float3;
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

pub(super) fn drain_for_transfusion(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
) -> Option<Option<f32>> {
    let health = health(spring, unit_id)?;
    if health.current >= health.max {
        return None;
    }
    let required = LIFE_BONUS + health.max - health.current;
    let (found, shortfall) = drain(spring, global, unit_id, required, true);
    if !found && shortfall.is_none() {
        return None;
    }
    Some(shortfall.map(|value| value - LIFE_BONUS))
}

pub(super) fn drain(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    collector_id: i32,
    required: f32,
    allow_partial: bool,
) -> (bool, Option<f32>) {
    let Some(origin) = units::position(spring, collector_id) else {
        return (false, None);
    };
    let house = defs(spring).id(UnitKind::House);
    let mut remaining = required;
    let mut to_kill: Vec<i32> = Vec::new();
    let mut to_restore: Vec<i32> = Vec::new();
    for target in nearest_allies(spring, collector_id, origin)
        .into_iter()
        .take(DRAIN_SEARCH_LIMIT)
    {
        if def_of(spring, target) == house {
            transfer(spring, target, NEUTRAL_TEAM);
            to_restore.push(target);
            continue;
        }
        let Some(health) = health(spring, target) else {
            break;
        };
        let available = health.current + LIFE_BONUS;
        if available > remaining {
            drain_effect(spring, target);
            blood_trail(spring, origin, target);
            crate::engine::check(
                spring,
                spring
                    .unit_control()
                    .set_unit_health(target, health_value(available - remaining)),
                "set_unit_health",
                target,
            );
            restore(spring, &to_restore);
            record_drained(global, consume(spring, &to_kill, origin));
            return (true, None);
        }
        to_kill.push(target);
        transfer(spring, target, NEUTRAL_TEAM);
        remaining -= available;
    }
    restore(spring, &to_restore);
    if allow_partial && !to_kill.is_empty() {
        record_drained(global, consume(spring, &to_kill, origin));
        return (false, Some(remaining));
    }
    restore(spring, &to_kill);
    (false, None)
}

fn nearest_allies(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    collector_id: i32,
    origin: Float3,
) -> Vec<i32> {
    let Ok(ally_team) = spring.units_info().get_unit_ally_team(collector_id) else {
        return Vec::new();
    };
    let range_squared = BLOOD_MAGIC_RANGE * BLOOD_MAGIC_RANGE;
    let mut allies: Vec<(f32, i32)> = units::units_in_cylinder(
        spring,
        origin.x,
        origin.z,
        BLOOD_MAGIC_RANGE + MIDDLE_MARGIN,
        ALL_UNITS,
    )
    .into_iter()
    .filter(|unit_id| {
        *unit_id != collector_id
            && spring.units_info().get_unit_ally_team(*unit_id).ok() == Some(ally_team)
    })
    .filter_map(|unit_id| {
        let middle = spring
            .units_info()
            .get_unit_position(
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

fn consume(spring: &spring::Spring<'_, spring::RulesSynced>, units: &[i32], origin: Float3) -> i32 {
    let mut drained = 0;
    for unit_id in units {
        let unit_id = *unit_id;
        drain_effect(spring, unit_id);
        blood_trail(spring, origin, unit_id);
        let options = spring::unit_control::DestroyUnitOptions {
            selfd: true,
            reclaimed: false,
            attacker_id: -1,
            recycle_id: false,
        };
        crate::with_addon_context(|ctx| ctx.commands().destroy_unit(unit_id, options)).then(
            move |spring, result| crate::engine::check(spring, result, "destroy_unit", unit_id),
        );
        drained += 1;
    }
    drained
}

fn drain_effect(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    spawn_effect(spring, unit_id, CEG_FLAMES);
    play_sound(spring, SOUND_DRAIN, SOUND_DRAIN_VOLUME, unit_id);
}

fn record_drained(global: &Global, count: i32) {
    global.score.access_mut().allies_drained += count;
}

fn restore(spring: &spring::Spring<'_, spring::RulesSynced>, units: &[i32]) {
    for unit_id in units {
        transfer(spring, *unit_id, PLAYER_TEAM);
    }
}

fn transfer(_spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32, team: i32) {
    crate::with_addon_context(|ctx| ctx.commands().transfer_unit(unit_id, team, false, false))
        .then(move |spring, result| crate::engine::check(spring, result, "transfer_unit", unit_id));
}

fn blood_trail(spring: &spring::Spring<'_, spring::RulesSynced>, origin: Float3, target_id: i32) {
    let Some(target) = units::position(spring, target_id) else {
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
            spring,
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
