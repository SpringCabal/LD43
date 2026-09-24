//! Transfusion and Migraine, cast through the input path: the rules message
//! the UI sends for keys 1 and 4, which the player takes on its next frame.

use alloc::format;
use alloc::vec::Vec;

use springcabal::message::Encoder;

use crate::defs::UnitKind;
use crate::units::{self, ENEMY_TEAM};

use super::*;

pub(super) const TRANSFUSION_CAST: i32 = 240;
pub(super) const TRANSFUSION_CHECK: i32 = 262;
/// Migraine is cast right after the Transfusion check.
pub(super) const MIGRAINE_CHECK: i32 = 292;
/// The blood mage is hurt down to this share of its health first.
const HURT_TO: f32 = 0.5;
/// Where the Migraine targets stand, from the blood mage.
const MIGRAINE_OFFSET: f32 = 250.0;
const MIGRAINE_TARGETS: usize = 2;
const PAUSED_RELOAD: f32 = 100000.0;

pub(super) fn cast_transfusion(world: &World) {
    let Some((mage, position)) = mage(world) else {
        world.track(|tracker| check(tracker, false, "the blood mage is there for Transfusion"));
        return;
    };
    let hurt = units::health(mage).map(|health| health.max * HURT_TO);
    let hurt_ok =
        hurt.is_some_and(|hurt| spring::set_unit_health(mage, units::health_value(hurt)).is_ok());
    let drained = world.global.score.access().allies_drained;
    let sent = send_spell(1, position);
    world.track(|tracker| {
        check(
            tracker,
            hurt_ok && sent,
            &format!("Transfusion is sent as the UI sends it, to a hurt mage ({hurt:?} health)"),
        );
        tracker.drained_before_transfusion = drained;
    });
}

pub(super) fn transfusion_checks(tracker: &mut Tracker, world: &World) {
    let health = world
        .global
        .player
        .access()
        .unit_id()
        .and_then(units::health);
    let healed = health
        .as_ref()
        .is_some_and(|health| health.current >= health.max - 1.0);
    check(
        tracker,
        healed,
        &format!(
            "Transfusion heals the blood mage fully ({:?} of {:?})",
            health.as_ref().map(|health| health.current),
            health.as_ref().map(|health| health.max)
        ),
    );
    let drained = world.global.score.access().allies_drained;
    check(
        tracker,
        drained > tracker.drained_before_transfusion,
        &format!(
            "Transfusion pays with villagers ({} -> {drained} drained)",
            tracker.drained_before_transfusion
        ),
    );
}

pub(super) fn cast_migraine(world: &World) {
    let Some((_, position)) = mage(world) else {
        world.track(|tracker| check(tracker, false, "the blood mage is there for Migraine"));
        return;
    };
    let (x, z) = (position.x + MIGRAINE_OFFSET, position.z);
    let targets: Vec<i32> = (0..MIGRAINE_TARGETS)
        .filter_map(|index| {
            units::create(
                UnitKind::OrkSmall,
                x,
                z + 40.0 * index as f32,
                ENEMY_TEAM,
                0,
            )
        })
        .collect();
    let target = spring::types::Float3 {
        x,
        y: units::ground_height(x, z),
        z,
    };
    let sent = send_spell(4, target);
    world.track(|tracker| {
        check(
            tracker,
            sent && targets.len() == MIGRAINE_TARGETS,
            &format!(
                "Migraine is sent as the UI sends it, at {} orks",
                targets.len()
            ),
        );
        tracker.migraine_targets = targets;
    });
}

pub(super) fn migraine_checks(world: &World) {
    let targets = world.track(|tracker| core::mem::take(&mut tracker.migraine_targets));
    let paused = targets
        .iter()
        .filter(|unit| {
            let reload =
                spring::owned::units_weapons::get_unit_weapon_state(**unit, 1, "reloadTime")
                    .map(|state| state.reload_time)
                    .unwrap_or(0.0);
            reload >= PAUSED_RELOAD
        })
        .count();
    world.track(|tracker| {
        check(
            tracker,
            paused == MIGRAINE_TARGETS,
            &format!("Migraine stuns the orks around its target ({paused} of {MIGRAINE_TARGETS})"),
        )
    });
    // Out of the way of the ork advance check.
    for unit in targets {
        units::destroy(unit);
    }
}

fn mage(world: &World) -> Option<(i32, spring::types::Float3)> {
    let mage = world.global.player.access().unit_id()?;
    Some((mage, units::position(mage)?))
}

/// Sends the UI's `spell|index|x|y|z` message.
fn send_spell(index: i32, target: spring::types::Float3) -> bool {
    let message = Encoder::new("spell")
        .field(index)
        .field(target.x)
        .field(target.y)
        .field(target.z)
        .finish();
    spring::send_lua_rules_msg(&message).unwrap_or(false)
}

pub(super) const VICTORY_ATTACK: i32 = 405;
pub(super) const VICTORY_ATTACK_CHECK: i32 = 420;

/// After victory the UI's attack is ignored, as Lua's player controller
/// ignores every message then.
pub(super) fn attack_after_victory(world: &World) {
    let Some((_, position)) = mage(world) else {
        return;
    };
    let message = Encoder::new("attack")
        .field(position.x + 200.0)
        .field(position.z)
        .finish();
    let sent = spring::send_lua_rules_msg(&message).unwrap_or(false);
    world.track(|tracker| tracker.victory_attack_sent = sent);
}

pub(super) fn attack_after_victory_checks(tracker: &mut Tracker, world: &World) {
    let target = world
        .global
        .player
        .access()
        .unit_id()
        .and_then(crate::unit_scripts::fire_target);
    check(
        tracker,
        tracker.victory_attack_sent && target == Some(None),
        &format!("after victory the blood mage ignores an attack order (fire target {target:?})"),
    );
}
