use alloc::format;
use alloc::vec::Vec;

use spring::cmd;

use crate::combat;
use crate::defs::UnitKind;
use crate::units::{self, ENEMY_TEAM, PLAYER_TEAM};

use super::*;

const FIXTURE_X: f32 = 8500.0;
const FIXTURE_Z: f32 = 8500.0;
const ORK_OFFSET: f32 = 40.0;
pub(super) const STUN_SETUP: i32 = 160;
pub(super) const STUN_START: i32 = 190;
const STUN_FRAMES: i32 = 90;
const STUN_SETTLE: i32 = 5;
pub(super) const STUN_PAUSED_CHECK: i32 = 230;
const STUN_END: i32 = STUN_START + STUN_FRAMES + 1;
pub(super) const STUN_OVER_CHECK: i32 = 360;
const PAUSED_RELOAD: f32 = 100000.0;

#[derive(Clone, Copy)]
pub struct Bite {
    pub attacker: i32,
    pub frame: i32,
}

pub(super) fn stun_fixture(_spring: &spring::Spring<'_, spring::RulesSynced>, world: &World) {
    let tracker = Rc::clone(world.tracker);
    crate::with_addon_context(|ctx| {
        ctx.commands().queue(|spring| {
            let peasant = units::create(
                spring,
                UnitKind::Peasant,
                FIXTURE_X,
                FIXTURE_Z,
                PLAYER_TEAM,
                0,
            );
            let ork = units::create(
                spring,
                UnitKind::OrkSmall,
                FIXTURE_X + ORK_OFFSET,
                FIXTURE_Z,
                ENEMY_TEAM,
                0,
            );
            (peasant, ork)
        })
    })
    .then(move |spring, fixture| {
        let (Some(peasant), Some(ork)) = fixture else {
            check(
                spring,
                &mut tracker.borrow_mut(),
                false,
                "the stunned-ork fixture spawned",
            );
            return;
        };
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .give_order_to_unit(ork, cmd::ATTACK, &[peasant as f32], 0, 0)
        })
        .then(move |spring, ordered| {
            let mut tracker = tracker.borrow_mut();
            check(
                spring,
                &mut tracker,
                ordered.is_ok(),
                &format!("the fixture ork is ordered to attack a peasant ({ordered:?})"),
            );
            tracker.stun_ork = ork;
        });
    });
}

pub(super) fn stun_the_ork(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    let ork = tracker.stun_ork;
    let bites = bites_between(tracker, 0, world.frame);
    check(
        spring,
        tracker,
        bites > 0,
        &format!("the fixture ork bites before its stun ({bites} bites)"),
    );
    combat::stun(spring, world.global, ork, STUN_FRAMES, world.frame);
}

pub(super) fn stunned_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    _world: &World,
) {
    let reloads = reloads(spring, tracker.stun_ork);
    check(
        spring,
        tracker,
        reloads.len() == 2 && reloads.iter().all(|reload| *reload >= PAUSED_RELOAD),
        &format!("a stun pauses every weapon of the ork, the firebomb too (reloads {reloads:?})"),
    );
}

pub(super) fn stun_over_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    _world: &World,
) {
    let stunned = bites_between(tracker, STUN_START + STUN_SETTLE, STUN_END);
    check(
        spring,
        tracker,
        stunned == 0,
        &format!("a stunned ork doesn't bite ({stunned} bites while stunned)"),
    );
    let reloads = reloads(spring, tracker.stun_ork);
    check(
        spring,
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
        spring,
        tracker,
        after > 0,
        &format!("the ork bites again after its stun ({after} bites)"),
    );
}

const BITE_RELOAD: f32 = 0.5;
const FIREBOMB_RELOAD: f32 = 15.0;

fn bites_between(tracker: &Tracker, from: i32, to: i32) -> usize {
    tracker
        .bites
        .iter()
        .filter(|bite| bite.attacker == tracker.stun_ork && (from..to).contains(&bite.frame))
        .count()
}

fn reloads(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> Vec<f32> {
    let count = spring
        .units_weapons()
        .get_unit_weapon_count(unit_id)
        .unwrap_or(0) as i32;
    (1..=count)
        .filter_map(|weapon| {
            spring
                .units_weapons()
                .get_unit_weapon_state(unit_id, weapon, "reloadTime")
                .ok()
                .map(|state| state.reload_time)
        })
        .collect()
}
