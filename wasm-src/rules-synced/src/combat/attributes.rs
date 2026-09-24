use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spring::typed::MoveTypeNumericField;

use crate::defs::{DefInfo, defs};
use crate::state::Global;

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

pub fn add_effect(global: &Global, unit_id: i32, effect: Effect, modifier: Modifier) {
    {
        let mut attributes = global.attributes.access_mut();
        let effects = attributes.effects.entry(unit_id).or_default();
        effects.retain(|(existing, _)| *existing != effect);
        effects.push((effect, modifier));
    }
    apply(global, unit_id);
}

pub fn remove_effect(global: &Global, unit_id: i32, effect: Effect) {
    if let Some(effects) = global.attributes.access_mut().effects.get_mut(&unit_id) {
        effects.retain(|(existing, _)| *existing != effect);
    }
    apply(global, unit_id);
}

pub fn forget(global: &Global, unit_id: i32) {
    let mut attributes = global.attributes.access_mut();
    attributes.effects.remove(&unit_id);
    attributes.base_reloads.remove(&unit_id);
}

const BASE_MOVE: f32 = 1.0;
const BASE_RELOAD: f32 = 1.0;
const MINIMUM_MOVE: f32 = 0.001;
// Long enough that a paused weapon never comes off cooldown in a match.
const PAUSED_RELOAD: f32 = 100000.0;

/// The attribute effects on every unit, like the Lua build's GG.Attributes.
///
/// Shared: spells, the ork AI, the combat gadget and the unit scripts' Killed
/// all add effects. It is never borrowed across an engine call.
#[derive(Default)]
pub struct Attributes {
    /// Each unit's effects in the order they were added, which is the order
    /// their factors multiply in.
    effects: BTreeMap<i32, Vec<(Effect, Modifier)>>,
    /// Each unit's unmodified reload; see `base_reload`.
    base_reloads: BTreeMap<i32, f32>,
}

fn apply(global: &Global, unit_id: i32) {
    let (move_factor, reload_factor) = combined(&global.attributes.access(), unit_id);
    apply_movement(unit_id, move_factor);
    apply_reload(global, unit_id, reload_factor);
}

fn combined(attributes: &Attributes, unit_id: i32) -> (f32, f32) {
    let mut move_factor = BASE_MOVE;
    let mut reload_factor = BASE_RELOAD;
    for (_, modifier) in attributes.effects.get(&unit_id).into_iter().flatten() {
        move_factor *= modifier.move_factor;
        reload_factor *= modifier.reload_factor;
    }
    (move_factor, reload_factor)
}

fn apply_movement(unit_id: i32, factor: f32) {
    let Some(info) = def_info(unit_id) else {
        return;
    };
    let Some(base) = info.speed else {
        return;
    };
    let factor = factor.max(0.0);
    let _ = spring::typed::set_move_type_numeric(
        unit_id,
        MoveTypeNumericField::MoveTypeMaxSpeed,
        base * factor,
    );
    let _ = spring::typed::set_move_type_numeric(
        unit_id,
        MoveTypeNumericField::MoveTypeMaxWantedSpeed,
        base * factor,
    );
    if factor <= 0.0 {
        let _ = spring::set_unit_velocity(unit_id, spring::types::Float3::default());
    }
    let _ = spring::typed::set_move_type_numeric(
        unit_id,
        MoveTypeNumericField::MoveTypeGroundTurnRate,
        info.turn_rate * factor.max(MINIMUM_MOVE),
    );
}

/// Scales the unit's reload time by `factor`.
///
/// A factor of zero means "never fire again", which the weapon state cannot
/// express directly, so it becomes a reload long enough to outlast any match.
fn apply_reload(global: &Global, unit_id: i32, factor: f32) {
    let Some(base) = base_reload(global, unit_id) else {
        return;
    };
    let reload = if factor <= 0.0 {
        PAUSED_RELOAD
    } else {
        base / factor
    };
    // Core UnitControl uses zero-based weapon slots. The read-only weapon
    // state query intentionally keeps Spring's Lua-compatible one-based
    // numbering, so the player's first weapon is slot 0 here.
    let _ = spring::set_unit_weapon_state(unit_id, 0, "reloadTime", reload);
}

/// The unit's unmodified reload, remembered the first time the unit is touched.
///
/// It has to be remembered rather than re-read: after the first application the
/// live weapon state already carries a factor, so deriving from it would
/// compound. The first call happens before any effect has been applied, so what
/// the weapon reports then is the def value.
fn base_reload(global: &Global, unit_id: i32) -> Option<f32> {
    let known = global
        .attributes
        .access()
        .base_reloads
        .get(&unit_id)
        .copied();
    if known.is_some() {
        return known;
    }
    let base = spring::owned::units_weapons::get_unit_weapon_state(unit_id, 1, "reloadTime")
        .ok()
        .map(|state| state.reload_time)
        .filter(|reload| *reload > 0.0)?;
    global
        .attributes
        .access_mut()
        .base_reloads
        .insert(unit_id, base);
    Some(base)
}

fn def_info(unit_id: i32) -> Option<DefInfo> {
    let unit_def_id = spring::units_info::get_unit_def_id(unit_id).ok()?;
    Some(defs().info(unit_def_id))
}
