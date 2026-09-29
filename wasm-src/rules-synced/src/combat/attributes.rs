//! Speed and reload effects on units: the Lua build's
//! `unit_attributes_generic.lua` (GG.Attributes), limited to the move and
//! reload factors TTLD's effects use, and `unit_turn_without_interia.lua`.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec::Vec;

use spring::ResultExt;
use spring::owned::units_weapons;
use spring::typed::MoveTypeNumericField;
use spring::types::Float3;

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

/// The attribute effects on every unit, like the Lua build's GG.Attributes.
///
/// Shared: spells, the ork AI, the combat gadget and the unit scripts' Killed
/// all add effects.
#[derive(Default)]
pub struct Attributes {
    units: BTreeMap<i32, UnitAttributes>,
    /// Units whose weapons are paused (a reload factor of 0); see
    /// [`update_paused`].
    paused: BTreeSet<i32>,
    /// Each unit def's unmodified weapons (Lua's `origUnitWeapons`).
    weapons: BTreeMap<i32, Vec<WeaponBase>>,
}

#[derive(Default)]
struct UnitAttributes {
    /// The effects in the order they were added, which is the order their
    /// factors multiply in.
    effects: Vec<(Effect, Modifier)>,
    /// The factors last applied; none yet means 1.
    applied_move: Option<f32>,
    applied_reload: Option<f32>,
}

#[derive(Clone, Copy)]
struct WeaponBase {
    /// Seconds.
    reload: f32,
    /// Seconds between the shots of a burst.
    burst_rate: f32,
}

pub fn add_effect(global: &Global, unit_id: i32, effect: Effect, modifier: Modifier) {
    {
        let mut attributes = global.attributes.access_mut();
        let effects = &mut attributes.units.entry(unit_id).or_default().effects;
        effects.retain(|(existing, _)| *existing != effect);
        effects.push((effect, modifier));
    }
    apply(global, unit_id);
}

pub fn remove_effect(global: &Global, unit_id: i32, effect: Effect) {
    {
        let mut attributes = global.attributes.access_mut();
        let Some(unit) = attributes.units.get_mut(&unit_id) else {
            return;
        };
        unit.effects.retain(|(existing, _)| *existing != effect);
    }
    apply(global, unit_id);
}

pub fn forget(global: &Global, unit_id: i32) {
    let mut attributes = global.attributes.access_mut();
    attributes.units.remove(&unit_id);
    attributes.paused.remove(&unit_id);
}

/// A new ground unit turns without inertia (`unit_turn_without_interia.lua`).
pub fn remove_turn_inertia(unit_id: i32) {
    let Some(movement) = movement_of(unit_id) else {
        return;
    };
    set_move(
        unit_id,
        MoveTypeNumericField::MoveTypeGroundTurnAccel,
        movement.turn_rate * TURN_ACCEL_PER_TURN_RATE,
        "turnAccel",
    );
}

/// Keeps paused weapons from reloading, every [`UPDATE_PERIOD`] frames, as
/// Lua's `UpdatePausedReload` does. Call it every frame.
pub fn update_paused(global: &Global, frame: i32) {
    if frame % UPDATE_PERIOD != 1 {
        return;
    }
    let paused: Vec<i32> = global.attributes.access().paused.iter().copied().collect();
    for unit_id in paused {
        // A dying unit pauses its weapons after UnitDestroyed has forgotten
        // it, so it is dropped here once it is gone.
        if !units::valid(unit_id) {
            global.attributes.access_mut().paused.remove(&unit_id);
            continue;
        }
        for weapon in 0..weapon_count(unit_id) {
            let Some(state) = weapon_state(unit_id, weapon) else {
                continue;
            };
            let reload_state = if state.reload_frame < 0.0 {
                (frame + UPDATE_PERIOD + 1) as f32
            } else {
                rescaled(frame, &state, PAUSED_RELOAD) + UPDATE_PERIOD as f32
            };
            set_weapon(unit_id, weapon, "reloadTime", PAUSED_RELOAD);
            set_weapon(unit_id, weapon, "reloadState", reload_state);
        }
    }
}

const UPDATE_PERIOD: i32 = 3;
/// Seconds. Long enough that a paused weapon never comes off cooldown, short
/// enough for the engine's frame count.
const PAUSED_RELOAD: f32 = 100000.0;
/// Slowed units brake at this multiple of their brake rate, so they drop to
/// their new speed at once instead of coasting.
const SLOWED_BRAKE_FACTOR: f32 = 1000.0;
const MINIMUM_FACTOR: f32 = 0.001;
const TURN_ACCEL_PER_TURN_RATE: f32 = 1.2;

/// Applies the unit's combined factors where they changed, as Lua's
/// `UpdateUnitAttributes`.
fn apply(global: &Global, unit_id: i32) {
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
        apply_movement(unit_id, move_factor);
    }
    if reload_factor != applied_reload {
        apply_reload(global, unit_id, reload_factor);
    }
    if let Some(unit) = global.attributes.access_mut().units.get_mut(&unit_id) {
        unit.applied_move = Some(move_factor);
        unit.applied_reload = Some(reload_factor);
    }
}

/// Lua's `UpdateMovementSpeed` for a ground unit, with the turn and
/// acceleration factors equal to the move factor, as every TTLD effect has
/// them.
fn apply_movement(unit_id: i32, factor: f32) {
    let Some(movement) = movement_of(unit_id) else {
        return;
    };
    let slowed = factor < 1.0;
    let speed_factor = factor.max(0.0);
    if speed_factor <= 0.0 {
        stop_if_grounded(unit_id);
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
        // Keeps a slowed unit from speeding up briefly on a new order.
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
        set_move(unit_id, field, value, name);
    }
}

/// A unit that can't move stops, unless it is in the air: a unit killed
/// mid-throw keeps flying.
fn stop_if_grounded(unit_id: i32) {
    let Some(position) = units::position(unit_id) else {
        return;
    };
    if units::ground_height(position.x, position.z) >= position.y {
        spring::set_unit_velocity(unit_id, Float3::default())
            .log_err(&format!("stopping unit {unit_id}"));
    }
}

/// Lua's `UpdateWeapons`: scales every weapon's reload by `factor`, keeping
/// the progress of the reload under way. A factor of 0 pauses the weapons.
fn apply_reload(global: &Global, unit_id: i32, factor: f32) {
    let Some(bases) = weapon_bases(global, unit_id) else {
        return;
    };
    let frame = spring::game::current_frame();
    for (weapon, base) in bases.iter().enumerate() {
        let weapon = weapon as i32;
        let Some(state) = weapon_state(unit_id, weapon) else {
            continue;
        };
        if factor <= 0.0 {
            // As in Lua, only the first weapon pauses now; the others do
            // within UPDATE_PERIOD frames, in update_paused.
            if !global.attributes.access_mut().paused.insert(unit_id) {
                continue;
            }
            let reload_state = if state.reload_frame < frame as f32 {
                // Ready to fire: keep it almost ready.
                (frame + UPDATE_PERIOD + 1) as f32
            } else {
                rescaled(frame, &state, PAUSED_RELOAD) + UPDATE_PERIOD as f32
            };
            set_weapon(unit_id, weapon, "reloadTime", PAUSED_RELOAD);
            set_weapon(unit_id, weapon, "reloadState", reload_state);
        } else {
            global.attributes.access_mut().paused.remove(&unit_id);
            let reload = base.reload / factor;
            set_weapon(unit_id, weapon, "reloadTime", reload);
            set_weapon(
                unit_id,
                weapon,
                "reloadState",
                rescaled(frame, &state, reload),
            );
            set_weapon(unit_id, weapon, "burstRate", base.burst_rate / factor);
        }
    }
}

/// The frame the weapon's reload finishes at once its reload time becomes
/// `reload`, keeping the fraction still to go.
fn rescaled(frame: i32, state: &units_weapons::UnitWeaponState, reload: f32) -> f32 {
    let frame = frame as f32;
    frame + (state.reload_frame - frame) * reload / state.reload_time
}

/// The unit def's unmodified weapons. Read from the first unit of the def,
/// before any effect has changed it, so they are the def's values.
fn weapon_bases(global: &Global, unit_id: i32) -> Option<Vec<WeaponBase>> {
    let def_id = units::def_of(unit_id)?;
    if let Some(bases) = global.attributes.access().weapons.get(&def_id) {
        return Some(bases.clone());
    }
    let bases: Vec<WeaponBase> = (0..weapon_count(unit_id))
        .map_while(|weapon| {
            weapon_state(unit_id, weapon).map(|state| WeaponBase {
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

fn weapon_count(unit_id: i32) -> i32 {
    units_weapons::get_unit_weapon_count(unit_id)
        .log_err(&format!("weapon count of unit {unit_id}"))
        .unwrap_or(0) as i32
}

/// The state of a weapon, by its zero-based slot. The read-only query counts
/// weapons from 1, as Lua does; the setters count from 0.
fn weapon_state(unit_id: i32, weapon: i32) -> Option<units_weapons::UnitWeaponState> {
    units_weapons::get_unit_weapon_state(unit_id, weapon + 1, "reloadTime")
        .log_err(&format!("weapon {weapon} of unit {unit_id}"))
}

fn set_weapon(unit_id: i32, weapon: i32, key: &str, value: f32) {
    spring::set_unit_weapon_state(unit_id, weapon, key, value)
        .log_err(&format!("{key} of weapon {weapon} of unit {unit_id}"));
}

fn set_move(unit_id: i32, field: MoveTypeNumericField, value: f32, name: &str) {
    spring::typed::set_move_type_numeric(unit_id, field, value)
        .log_err(&format!("{name} of unit {unit_id}"));
}

fn movement_of(unit_id: i32) -> Option<Movement> {
    defs().info(units::def_of(unit_id)?).movement
}
