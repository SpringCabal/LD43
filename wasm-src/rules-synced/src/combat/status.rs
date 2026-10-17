use alloc::vec::Vec;

use crate::defs::{UnitKind, defs};
use crate::state::Global;
use crate::unit_scripts::ScriptCall;

use super::attributes::{self, Effect, Modifier};

#[derive(Default)]
pub struct Statuses {
    timers: Vec<Timer>,
}

impl Statuses {
    fn schedule(&mut self, unit_id: i32, effect: Effect, end_frame: i32) {
        self.timers
            .retain(|timer| timer.unit_id != unit_id || timer.effect != effect);
        self.timers.push(Timer {
            unit_id,
            effect,
            end_frame,
        });
    }

    fn take_expired(&mut self, frame: i32) -> Vec<Timer> {
        let expired = self
            .timers
            .iter()
            .copied()
            .filter(|timer| frame > timer.end_frame)
            .collect();
        self.timers.retain(|timer| frame <= timer.end_frame);
        expired
    }

    pub fn forget(&mut self, unit_id: i32) {
        self.timers.retain(|timer| timer.unit_id != unit_id);
    }
}

pub fn adrenaline(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    duration: i32,
    frame: i32,
) {
    let modifier = if resists(spring, unit_id) {
        Modifier {
            move_factor: 1.8,
            reload_factor: 2.5,
        }
    } else {
        Modifier {
            move_factor: 2.0,
            reload_factor: 2.5,
        }
    };
    attributes::add_effect(spring, global, unit_id, Effect::Buff, modifier);
    crate::unit_scripts::call(spring, unit_id, ScriptCall::Buff { frames: duration });
    global
        .statuses
        .access_mut()
        .schedule(unit_id, Effect::Buff, frame + duration);
}

pub fn stun(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    duration: i32,
    frame: i32,
) {
    let duration = if resists(spring, unit_id) {
        (duration as f32 * 0.5) as i32
    } else {
        duration
    };
    attributes::add_effect(
        spring,
        global,
        unit_id,
        Effect::Stun,
        Modifier {
            move_factor: 0.1,
            reload_factor: 0.0,
        },
    );
    crate::unit_scripts::call(spring, unit_id, ScriptCall::Stun { frames: duration });
    global
        .statuses
        .access_mut()
        .schedule(unit_id, Effect::Stun, frame + duration);
}

pub fn update(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, frame: i32) {
    let expired = global.statuses.access_mut().take_expired(frame);
    for timer in expired {
        attributes::remove_effect(spring, global, timer.unit_id, timer.effect);
    }
}

#[derive(Clone, Copy)]
struct Timer {
    unit_id: i32,
    effect: Effect,
    end_frame: i32,
}

fn resists(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> bool {
    let Ok(unit_def_id) = spring.units_info().get_unit_def_id(unit_id) else {
        return false;
    };
    matches!(
        defs(spring).kind(unit_def_id),
        Some(UnitKind::OrkBoss | UnitKind::BloodMage)
    )
}
