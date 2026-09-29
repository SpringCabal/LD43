//! A stunned ork: its weapons pause, and it attacks again once the stun
//! wears off. Lua's `UpdateWeapons` keeps the reload under way; scaling only
//! the reload time let a stunned ork fire once with a 100,000 s reload and
//! disarmed it for the rest of the round.

use alloc::format;
use alloc::vec::Vec;

use spring::cmd;
use spring::commands;

use crate::combat;
use crate::defs::UnitKind;
use crate::units::{self, ENEMY_TEAM, PLAYER_TEAM};

use super::*;

/// The fixture stands far from the town and the ork waves.
const FIXTURE_X: f32 = 8500.0;
const FIXTURE_Z: f32 = 8500.0;
const ORK_OFFSET: f32 = 40.0;
pub(super) const STUN_SETUP: i32 = 160;
pub(super) const STUN_START: i32 = 190;
const STUN_FRAMES: i32 = 90;
/// Bites already on their way when the stun lands still arrive.
const STUN_SETTLE: i32 = 5;
pub(super) const STUN_PAUSED_CHECK: i32 = 230;
/// The stun ends on the first frame after STUN_START + STUN_FRAMES.
const STUN_END: i32 = STUN_START + STUN_FRAMES + 1;
pub(super) const STUN_OVER_CHECK: i32 = 360;
/// The paused reload the attributes use, in seconds.
const PAUSED_RELOAD: f32 = 100000.0;

/// A bite: the ork that bit, and the frame.
#[derive(Clone, Copy)]
pub struct Bite {
    pub attacker: i32,
    pub frame: i32,
}

pub(super) fn stun_fixture(world: &World) {
    let tracker = Rc::clone(world.tracker);
    commands::queue(|| {
        let peasant = units::create(UnitKind::Peasant, FIXTURE_X, FIXTURE_Z, PLAYER_TEAM, 0);
        let ork = units::create(
            UnitKind::OrkSmall,
            FIXTURE_X + ORK_OFFSET,
            FIXTURE_Z,
            ENEMY_TEAM,
            0,
        );
        (peasant, ork)
    })
    .then(move |fixture| {
        let (Some(peasant), Some(ork)) = fixture else {
            check(
                &mut tracker.borrow_mut(),
                false,
                "the stunned-ork fixture spawned",
            );
            return;
        };
        commands::give_order_to_unit(ork, cmd::ATTACK, &[peasant as f32], 0, 0).then(
            move |ordered| {
                let mut tracker = tracker.borrow_mut();
                check(
                    &mut tracker,
                    ordered.is_ok(),
                    &format!("the fixture ork is ordered to attack a peasant ({ordered:?})"),
                );
                tracker.stun_ork = ork;
            },
        );
    });
}

pub(super) fn stun_the_ork(tracker: &mut Tracker, world: &World) {
    let ork = tracker.stun_ork;
    let bites = bites_between(tracker, 0, world.frame);
    check(
        tracker,
        bites > 0,
        &format!("the fixture ork bites before its stun ({bites} bites)"),
    );
    combat::stun(world.global, ork, STUN_FRAMES, world.frame);
}

pub(super) fn stunned_checks(tracker: &mut Tracker, _world: &World) {
    let reloads = reloads(tracker.stun_ork);
    check(
        tracker,
        reloads.len() == 2 && reloads.iter().all(|reload| *reload >= PAUSED_RELOAD),
        &format!("a stun pauses every weapon of the ork, the firebomb too (reloads {reloads:?})"),
    );
}

pub(super) fn stun_over_checks(tracker: &mut Tracker, _world: &World) {
    let stunned = bites_between(tracker, STUN_START + STUN_SETTLE, STUN_END);
    check(
        tracker,
        stunned == 0,
        &format!("a stunned ork doesn't bite ({stunned} bites while stunned)"),
    );
    let reloads = reloads(tracker.stun_ork);
    check(
        tracker,
        reloads.len() == 2
            && (reloads[0] - BITE_RELOAD).abs() < 0.01
            && (reloads[1] - FIREBOMB_RELOAD).abs() < 0.01,
        &format!(
            "after the stun the ork's weapons reload at their def rates (reloads {reloads:?})"
        ),
    );
    let after = bites_between(tracker, STUN_END, STUN_OVER_CHECK);
    check(
        tracker,
        after > 0,
        &format!("the ork bites again after its stun ({after} bites)"),
    );
}

/// Seconds, from the ork weapon defs.
const BITE_RELOAD: f32 = 0.5;
const FIREBOMB_RELOAD: f32 = 15.0;

fn bites_between(tracker: &Tracker, from: i32, to: i32) -> usize {
    tracker
        .bites
        .iter()
        .filter(|bite| bite.attacker == tracker.stun_ork && (from..to).contains(&bite.frame))
        .count()
}

/// The live reload of each of the unit's weapons, in seconds.
fn reloads(unit_id: i32) -> Vec<f32> {
    let count = spring::owned::units_weapons::get_unit_weapon_count(unit_id).unwrap_or(0) as i32;
    (1..=count)
        .filter_map(|weapon| {
            spring::owned::units_weapons::get_unit_weapon_state(unit_id, weapon, "reloadTime")
                .ok()
                .map(|state| state.reload_time)
        })
        .collect()
}
