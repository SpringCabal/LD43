use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec::Vec;

use spring::Float3;
use spring::ResultExt;
use spring::move_ctrl::MoveTypeNumericField;
use spring::units_weapons;

use crate::defs::{Movement, defs};
use crate::state::Global;
use crate::units;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Buff,
    Stun,
    HalfSpeed,
    Flee,
    DeathParalysis,
}

#[derive(Clone, Copy)]
pub struct Modifier {
    pub move_factor: f32,
    pub reload_factor: f32,
}

#[derive(Default)]
pub struct Attributes {
    units: BTreeMap<i32, UnitAttributes>,
    paused: BTreeSet<i32>,
    weapons: BTreeMap<i32, Vec<WeaponBase>>,
}

#[derive(Default)]
struct UnitAttributes {
    effects: Vec<(Effect, Modifier)>,
    applied_move: Option<f32>,
    applied_reload: Option<f32>,
}

#[derive(Clone, Copy)]
struct WeaponBase {
    range: f32,
    reload: f32,
    burst_rate: f32,
}

pub fn add_effect(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    effect: Effect,
    modifier: Modifier,
) {
    {
        let mut attributes = global.attributes.access_mut();
        let effects = &mut attributes.units.entry(unit_id).or_default().effects;
        effects.retain(|(existing, _)| *existing != effect);
        effects.push((effect, modifier));
    }
    apply(spring, global, unit_id);
}

pub fn remove_effect(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    effect: Effect,
) {
    {
        let mut attributes = global.attributes.access_mut();
        let Some(unit) = attributes.units.get_mut(&unit_id) else {
            return;
        };
        unit.effects.retain(|(existing, _)| *existing != effect);
    }
    apply(spring, global, unit_id);
}

pub fn forget(global: &Global, unit_id: i32) {
    let mut attributes = global.attributes.access_mut();
    attributes.units.remove(&unit_id);
    attributes.paused.remove(&unit_id);
}

pub fn remove_turn_inertia(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    let Some(movement) = movement_of(spring, unit_id) else {
        return;
    };
    set_move(
        spring,
        unit_id,
        MoveTypeNumericField::MoveTypeGroundTurnAccel,
        movement.turn_rate * TURN_ACCEL_PER_TURN_RATE,
        "turnAccel",
    );
}

pub fn update_paused(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    frame: i32,
) {
    if frame % UPDATE_PERIOD != 1 {
        return;
    }
    let paused: Vec<i32> = global.attributes.access().paused.iter().copied().collect();
    for unit_id in paused {
        if !units::valid(spring, unit_id) {
            global.attributes.access_mut().paused.remove(&unit_id);
            continue;
        }
        for weapon in 1..=weapon_count(spring, unit_id) {
            let Some(state) = weapon_state(spring, unit_id, weapon) else {
                continue;
            };
            let reload_state = if state.reload_frame < 0.0 {
                (frame + UPDATE_PERIOD + 1) as f32
            } else {
                rescaled(frame, &state, PAUSED_RELOAD) + UPDATE_PERIOD as f32
            };
            set_weapon(spring, unit_id, weapon, "reloadTime", PAUSED_RELOAD);
            set_weapon(spring, unit_id, weapon, "reloadState", reload_state);
        }
    }
}

const UPDATE_PERIOD: i32 = 3;
const PAUSED_RELOAD: f32 = 100000.0;
const SLOWED_BRAKE_FACTOR: f32 = 1000.0;
const MINIMUM_FACTOR: f32 = 0.001;
const TURN_ACCEL_PER_TURN_RATE: f32 = 1.2;

fn apply(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, unit_id: i32) {
    let (move_factor, reload_factor, applied_move, applied_reload) = {
        let attributes = global.attributes.access();
        let Some(unit) = attributes.units.get(&unit_id) else {
            return;
        };
        let (move_factor, reload_factor) =
            unit.effects
                .iter()
                .fold((1.0, 1.0), |(move_factor, reload_factor), (_, modifier)| {
                    (
                        move_factor * modifier.move_factor,
                        reload_factor * modifier.reload_factor,
                    )
                });
        (
            move_factor,
            reload_factor,
            unit.applied_move.unwrap_or(1.0),
            unit.applied_reload.unwrap_or(1.0),
        )
    };
    if move_factor != applied_move {
        apply_movement(spring, unit_id, move_factor);
    }
    if reload_factor != applied_reload {
        apply_reload(spring, global, unit_id, reload_factor);
    }
    if let Some(unit) = global.attributes.access_mut().units.get_mut(&unit_id) {
        unit.applied_move = Some(move_factor);
        unit.applied_reload = Some(reload_factor);
    }
}

fn apply_movement(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32, factor: f32) {
    let Some(movement) = movement_of(spring, unit_id) else {
        return;
    };
    let slowed = factor < 1.0;
    let speed_factor = factor.max(0.0);
    if speed_factor <= 0.0 {
        stop_if_grounded(spring, unit_id);
    }
    let turn_accel_factor = factor.max(0.0);
    let turn_factor = turn_accel_factor.max(MINIMUM_FACTOR);
    let accel_factor = if factor <= 0.0 {
        MINIMUM_FACTOR
    } else {
        factor
    };
    let brake_factor = if slowed {
        SLOWED_BRAKE_FACTOR
    } else {
        accel_factor
    };
    let mut acc_rate = movement.max_acc * accel_factor;
    if slowed && acc_rate > speed_factor {
        acc_rate = speed_factor;
    }
    use MoveTypeNumericField as Field;
    let fields = [
        (
            Field::MoveTypeMaxSpeed,
            movement.speed * speed_factor,
            "maxSpeed",
        ),
        (
            Field::MoveTypeGroundMaxReverseSpeed,
            if slowed { 0.0 } else { movement.reverse_speed },
            "maxReverseSpeed",
        ),
        (
            Field::MoveTypeGroundTurnRate,
            movement.turn_rate * turn_factor,
            "turnRate",
        ),
        (Field::MoveTypeGroundAccRate, acc_rate, "accRate"),
        (
            Field::MoveTypeGroundDecRate,
            movement.max_dec * brake_factor,
            "decRate",
        ),
        (
            Field::MoveTypeGroundTurnAccel,
            movement.turn_rate * turn_accel_factor * TURN_ACCEL_PER_TURN_RATE,
            "turnAccel",
        ),
    ];
    for (field, value, name) in fields {
        set_move(spring, unit_id, field, value, name);
    }
}

fn stop_if_grounded(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    let Some(position) = units::position(spring, unit_id) else {
        return;
    };
    if units::ground_height(spring, position.x, position.z) >= position.y {
        spring
            .unit_control()
            .set_unit_velocity(unit_id, Float3::default())
            .log_err(spring, &format!("stopping unit {unit_id}"));
    }
}

fn apply_reload(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    factor: f32,
) {
    let Some(bases) = weapon_bases(spring, global, unit_id) else {
        return;
    };
    // Lua UpdateWeapons restores the def range, including secondary weapons.
    // An ork's script initially uses its melee range; leaving that override
    // after an attribute update changes CommandAI movement and corpse paths.
    if let Some(def_id) = units::def_of(spring, unit_id) {
        spring
            .unit_control()
            .set_unit_max_range(unit_id, defs(spring).info(spring, def_id).max_weapon_range)
            .log_err(spring, &format!("maximum weapon range of unit {unit_id}"));
    }
    let frame = spring.game().current_frame().unwrap_or(0);
    for (weapon, base) in bases.iter().enumerate() {
        let weapon = weapon as i32 + 1;
        let Some(state) = weapon_state(spring, unit_id, weapon) else {
            continue;
        };
        if factor <= 0.0 {
            if global.attributes.access_mut().paused.insert(unit_id) {
                let reload_state = if state.reload_frame < frame as f32 {
                    (frame + UPDATE_PERIOD + 1) as f32
                } else {
                    rescaled(frame, &state, PAUSED_RELOAD) + UPDATE_PERIOD as f32
                };
                set_weapon(spring, unit_id, weapon, "reloadTime", PAUSED_RELOAD);
                set_weapon(spring, unit_id, weapon, "reloadState", reload_state);
            }
        } else {
            global.attributes.access_mut().paused.remove(&unit_id);
            let reload = base.reload / factor;
            set_weapon(spring, unit_id, weapon, "reloadTime", reload);
            set_weapon(
                spring,
                unit_id,
                weapon,
                "reloadState",
                rescaled(frame, &state, reload),
            );
            set_weapon(
                spring,
                unit_id,
                weapon,
                "burstRate",
                base.burst_rate / factor,
            );
        }
        set_weapon(spring, unit_id, weapon, "range", base.range);
        spring
            .unit_control()
            .set_unit_weapon_damages(unit_id, weapon, "dynDamageRange", base.range)
            .log_err(
                spring,
                &format!("dynamic damage range of unit {unit_id} weapon {weapon}"),
            );
    }
}

fn rescaled(frame: i32, state: &units_weapons::UnitWeaponState, reload: f32) -> f32 {
    let frame = frame as f32;
    frame + (state.reload_frame - frame) * reload / state.reload_time
}

fn weapon_bases(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
) -> Option<Vec<WeaponBase>> {
    let def_id = units::def_of(spring, unit_id)?;
    if let Some(bases) = global.attributes.access().weapons.get(&def_id) {
        return Some(bases.clone());
    }
    let bases: Vec<WeaponBase> = (1..=weapon_count(spring, unit_id))
        .map_while(|weapon| {
            weapon_state(spring, unit_id, weapon).map(|state| WeaponBase {
                range: state.range,
                reload: state.reload_time,
                burst_rate: state.salvo_delay,
            })
        })
        .collect();
    global
        .attributes
        .access_mut()
        .weapons
        .insert(def_id, bases.clone());
    Some(bases)
}

fn weapon_count(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> i32 {
    spring
        .units_weapons()
        .get_unit_weapon_count(unit_id)
        .log_err(spring, &format!("weapon count of unit {unit_id}"))
        .unwrap_or(0) as i32
}

fn weapon_state(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    weapon: i32,
) -> Option<units_weapons::UnitWeaponState> {
    spring
        .units_weapons()
        .get_unit_weapon_state(unit_id, weapon, "reloadTime")
        .log_err(spring, &format!("weapon {weapon} of unit {unit_id}"))
}

fn set_weapon(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    weapon: i32,
    key: &str,
    value: f32,
) {
    spring
        .unit_control()
        .set_unit_weapon_state(unit_id, weapon, key, value)
        .log_err(
            spring,
            &format!("{key} of weapon {weapon} of unit {unit_id}"),
        );
}

fn set_move(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    field: MoveTypeNumericField,
    value: f32,
    name: &str,
) {
    spring
        .move_ctrl()
        .set_move_type_numeric(unit_id, field, value)
        .log_err(spring, &format!("{name} of unit {unit_id}"));
}

fn movement_of(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> Option<Movement> {
    defs(spring)
        .info(spring, units::def_of(spring, unit_id)?)
        .movement
}
