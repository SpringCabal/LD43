use alloc::vec::Vec;

use spring::typed::MoveTypeNumericField;

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

pub fn add_effect(unit_id: i32, effect: Effect, modifier: Modifier) {
    with_effects(|effects| {
        effects.retain(|entry| entry.unit_id != unit_id || entry.effect != effect);
        effects.push(Entry {
            unit_id,
            effect,
            modifier,
        });
    });
    apply(unit_id);
}

pub fn remove_effect(unit_id: i32, effect: Effect) {
    with_effects(|effects| {
        effects.retain(|entry| entry.unit_id != unit_id || entry.effect != effect);
    });
    apply(unit_id);
}

pub fn forget(unit_id: i32) {
    with_effects(|effects| effects.retain(|entry| entry.unit_id != unit_id));
    let _ = BASE_RELOADS.with(|reloads| {
        reloads
            .try_borrow_mut()
            .map(|mut reloads| reloads.retain(|(id, _)| *id != unit_id))
    });
}

const BASE_MOVE: f32 = 1.0;
const BASE_RELOAD: f32 = 1.0;
const MINIMUM_MOVE: f32 = 0.001;
// Long enough that a paused weapon never comes off cooldown in a match.
const PAUSED_RELOAD: f32 = 100000.0;

#[derive(Clone, Copy)]
struct Entry {
    unit_id: i32,
    effect: Effect,
    modifier: Modifier,
}

thread_local! {
    static BASE_RELOADS: core::cell::RefCell<Vec<(i32, f32)>> = const {
        core::cell::RefCell::new(Vec::new())
    };
    static EFFECTS: core::cell::RefCell<Vec<Entry>> = const {
        core::cell::RefCell::new(Vec::new())
    };
}

fn with_effects<R>(function: impl FnOnce(&mut Vec<Entry>) -> R) -> Option<R> {
    EFFECTS.with(|effects| effects.try_borrow_mut().ok().map(|mut e| function(&mut e)))
}

fn apply(unit_id: i32) {
    let Some((move_factor, reload_factor)) = combined(unit_id) else {
        return;
    };
    apply_movement(unit_id, move_factor);
    apply_reload(unit_id, reload_factor);
}

fn combined(unit_id: i32) -> Option<(f32, f32)> {
    with_effects(|effects| {
        let mut move_factor = BASE_MOVE;
        let mut reload_factor = BASE_RELOAD;
        for entry in effects.iter().filter(|entry| entry.unit_id == unit_id) {
            move_factor *= entry.modifier.move_factor;
            reload_factor *= entry.modifier.reload_factor;
        }
        (move_factor, reload_factor)
    })
}

fn apply_movement(unit_id: i32, factor: f32) {
    let Some(base) = base_speed(unit_id) else {
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
        turn_rate(unit_id) * factor.max(MINIMUM_MOVE),
    );
}

/// Scales the unit's reload time by `factor`.
///
/// A factor of zero means "never fire again", which the weapon state cannot
/// express directly, so it becomes a reload long enough to outlast any match.
fn apply_reload(unit_id: i32, factor: f32) {
    let Some(base) = base_reload(unit_id) else {
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
fn base_reload(unit_id: i32) -> Option<f32> {
    BASE_RELOADS.with(|reloads| {
        let mut reloads = reloads.try_borrow_mut().ok()?;
        if let Some((_, base)) = reloads.iter().find(|(id, _)| *id == unit_id) {
            return Some(*base);
        }
        let base = spring::owned::units_weapons::get_unit_weapon_state(unit_id, 1, "reloadTime")
            .ok()
            .map(|state| state.reload_time)
            .filter(|reload| *reload > 0.0)?;
        reloads.push((unit_id, base));
        Some(base)
    })
}

fn base_speed(unit_id: i32) -> Option<f32> {
    let unit_def_id = spring::units_info::get_unit_def_id(unit_id).ok()?;
    spring::get_unit_def_speed(unit_def_id).ok()
}

fn turn_rate(unit_id: i32) -> f32 {
    spring::units_info::get_unit_def_id(unit_id)
        .ok()
        .and_then(|unit_def_id| spring::get_unit_def_param_float(unit_def_id, "turnRate").ok())
        .unwrap_or(0.0)
}
