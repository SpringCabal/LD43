use alloc::vec::Vec;

use crate::units::{self, BLOOD_MAGE, OrkKind};

use super::attributes::{self, Effect, Modifier};

pub fn adrenaline(unit_id: i32, duration: i32, frame: i32) {
    let modifier = if resists(unit_id) {
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
    attributes::add_effect(unit_id, Effect::Buff, modifier);
    call_script(unit_id, "Buff", duration as f32);
    schedule(unit_id, Effect::Buff, frame + duration);
}

pub fn stun(unit_id: i32, duration: i32, frame: i32) {
    let duration = if resists(unit_id) {
        (duration as f32 * 0.5) as i32
    } else {
        duration
    };
    attributes::add_effect(
        unit_id,
        Effect::Stun,
        Modifier {
            move_factor: 0.1,
            reload_factor: 0.0,
        },
    );
    call_script(unit_id, "Stun", duration as f32);
    schedule(unit_id, Effect::Stun, frame + duration);
}

pub fn update(frame: i32) {
    let expired = with_timers(|timers| {
        let expired: Vec<Timer> = timers
            .iter()
            .copied()
            .filter(|timer| frame > timer.end_frame)
            .collect();
        timers.retain(|timer| frame <= timer.end_frame);
        expired
    })
    .unwrap_or_default();
    for timer in expired {
        attributes::remove_effect(timer.unit_id, timer.effect);
    }
}

pub fn forget(unit_id: i32) {
    let _ = with_timers(|timers| timers.retain(|timer| timer.unit_id != unit_id));
}

#[derive(Clone, Copy)]
struct Timer {
    unit_id: i32,
    effect: Effect,
    end_frame: i32,
}

thread_local! {
    static TIMERS: core::cell::RefCell<Vec<Timer>> = const {
        core::cell::RefCell::new(Vec::new())
    };
}

fn with_timers<R>(function: impl FnOnce(&mut Vec<Timer>) -> R) -> Option<R> {
    TIMERS.with(|timers| timers.try_borrow_mut().ok().map(|mut t| function(&mut t)))
}

fn schedule(unit_id: i32, effect: Effect, end_frame: i32) {
    let _ = with_timers(|timers| {
        timers.retain(|timer| timer.unit_id != unit_id || timer.effect != effect);
        timers.push(Timer {
            unit_id,
            effect,
            end_frame,
        });
    });
}

fn resists(unit_id: i32) -> bool {
    let Ok(unit_def_id) = spring::units_info::get_unit_def_id(unit_id) else {
        return false;
    };
    [
        units::def_id(OrkKind::Boss.def_name()),
        units::def_id(BLOOD_MAGE),
    ]
    .into_iter()
    .flatten()
    .any(|id| id == unit_def_id)
}

fn call_script(unit_id: i32, function_name: &str, argument: f32) {
    crate::unit_scripts::queue_call(unit_id, function_name, &[argument]);
}
