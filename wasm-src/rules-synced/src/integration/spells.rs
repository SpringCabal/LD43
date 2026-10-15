use alloc::format;
use alloc::vec::Vec;

use spring::ResultExt;
use spring::commands;
use springcabal::message::Encoder;
use ttld_shared::Command;

use crate::defs::UnitKind;
use crate::units::{self, ENEMY_TEAM, PLAYER_TEAM};

use super::*;

pub(super) const ADRENALINE_CAST: i32 = 120;
pub(super) const POULTICE_CAST: i32 = 130;
pub(super) const HEARTBURN_CAST: i32 = 150;
pub(super) const HEARTBURN_ALLY: i32 = 155;
pub(super) const SPELLS_LANDED_CHECK: i32 = 185;
pub(super) const TRANSFUSION_CAST: i32 = 240;
pub(super) const TRANSFUSION_CHECK: i32 = 262;
pub(super) const MIGRAINE_CHECK: i32 = 292;
const HURT_TO: f32 = 0.5;
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
    let sent = send_spell(TRANSFUSION, position);
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
    let tracker = Rc::clone(world.tracker);
    commands::queue(move || {
        (0..MIGRAINE_TARGETS)
            .filter_map(|index| {
                units::create(
                    UnitKind::OrkSmall,
                    x,
                    z + 40.0 * index as f32,
                    ENEMY_TEAM,
                    0,
                )
            })
            .collect::<Vec<i32>>()
    })
    .then(move |targets| {
        let target = spring::types::Float3 {
            x,
            y: units::ground_height(x, z),
            z,
        };
        let sent = send_spell(MIGRAINE, target);
        let mut tracker = tracker.borrow_mut();
        check(
            &mut tracker,
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
    for unit in targets {
        units::destroy(unit);
    }
}

pub(super) fn cast_adrenaline(world: &World) {
    let Some((_, position)) = mage(world) else {
        world.track(|tracker| check(tracker, false, "the blood mage is there for Adrenaline"));
        return;
    };
    let drained = world.global.score.access().allies_drained;
    let sent = send_spell(ADRENALINE, position);
    world.track(|tracker| {
        tracker.spell_sent = sent;
        tracker.drained_before_adrenaline = drained;
    });
}

pub(super) fn cast_poultice(world: &World) {
    let taken = cast_taken(world, ADRENALINE_CAST);
    let drained = world.global.score.access().allies_drained;
    world.track(|tracker| {
        check(
            tracker,
            tracker.spell_sent && taken,
            "Adrenaline, sent as the UI sends it, is cast",
        );
        check(
            tracker,
            drained > tracker.drained_before_adrenaline,
            &format!(
                "blood magic consumed villagers ({} -> {drained} drained)",
                tracker.drained_before_adrenaline
            ),
        );
    });
    let Some((_, position)) = mage(world) else {
        world.track(|tracker| check(tracker, false, "the blood mage is there for Poultice"));
        return;
    };
    let tracker = Rc::clone(world.tracker);
    units::spawn(
        UnitKind::OrkSmall,
        position.x + POULTICE_TARGET_OFFSET,
        position.z,
        ENEMY_TEAM,
        0,
    )
    .then(move |target| {
        let before = target.and_then(units::position);
        let sent = send_spell(POULTICE, position);
        let mut tracker = tracker.borrow_mut();
        tracker.spell_sent = sent;
        tracker.poultice_target = target.filter(|_| before.is_some()).unwrap_or(-1);
        if let Some(before) = before {
            tracker.poultice_x = before.x;
            tracker.poultice_z = before.z;
        }
        tracker.poultice_origin_x = position.x;
        tracker.poultice_origin_z = position.z;
    });
}

pub(super) fn cast_heartburn(world: &World) {
    let taken = cast_taken(world, POULTICE_CAST);
    world.track(|tracker| {
        check(
            tracker,
            tracker.spell_sent && taken,
            "Poultice, sent as the UI sends it, is cast",
        )
    });
    let Some((_, position)) = mage(world) else {
        world.track(|tracker| check(tracker, false, "the blood mage is there for Heartburn"));
        return;
    };
    let (tx, tz) = (position.x + 100.0, position.z);
    let tracker = Rc::clone(world.tracker);
    commands::queue(move || {
        let target = units::create(UnitKind::OrkSmall, tx, tz, ENEMY_TEAM, 0);
        let edge = units::create(
            UnitKind::OrkSmall,
            tx,
            tz + HEARTBURN_EDGE_OFFSET,
            ENEMY_TEAM,
            0,
        );
        (target, edge)
    })
    .then(move |(target, edge)| {
        for unit in [target, edge].into_iter().flatten() {
            spring::set_unit_neutral(unit, true).log_err("neutral Heartburn target");
        }
        let ground = units::ground_height(tx, tz);
        let sent = send_spell(
            HEARTBURN,
            spring::types::Float3 {
                x: tx,
                y: ground,
                z: tz,
            },
        );
        let mut tracker = tracker.borrow_mut();
        tracker.spell_sent = sent && target.is_some() && edge.is_some();
        tracker.heartburn_target = target.unwrap_or(-1);
        tracker.heartburn_edge = edge.unwrap_or(-1);
        tracker.heartburn_ground = (tx, tz, ground);
    });
}

pub(super) fn heartburn_ally(world: &World) {
    let taken = cast_taken(world, HEARTBURN_CAST);
    world.track(|tracker| {
        check(
            tracker,
            tracker.spell_sent && taken,
            "Heartburn, sent as the UI sends it, is cast with its targets in place",
        )
    });
    let (tx, tz, _) = world.track(|tracker| tracker.heartburn_ground);
    let tracker = Rc::clone(world.tracker);
    units::spawn(
        UnitKind::Peasant,
        tx,
        tz - HEARTBURN_EDGE_OFFSET,
        PLAYER_TEAM,
        0,
    )
    .then(move |ally| tracker.borrow_mut().heartburn_ally = ally.unwrap_or(-1));
}

const HEARTBURN_EDGE_OFFSET: f32 = 170.0;
const POULTICE_TARGET_OFFSET: f32 = 600.0;

pub(super) fn spells_landed_checks(tracker: &mut Tracker, _world: &World) {
    let damage_to = |unit_id: i32| {
        tracker
            .fireball_hits
            .iter()
            .filter(|(hit, _)| *hit == unit_id && unit_id >= 0)
            .map(|(_, damage)| *damage)
            .fold(0.0, f32::max)
    };
    let centre = damage_to(tracker.heartburn_target);
    let edge = damage_to(tracker.heartburn_edge);
    let ally = damage_to(tracker.heartburn_ally);
    check(
        tracker,
        centre > 0.0,
        &format!("heartburn's fireball explodes on its target ({centre:.0} damage)"),
    );
    let (tx, tz, _) = tracker.heartburn_ground;
    let edge_distance = units::position(tracker.heartburn_edge).map_or(-1.0, |position| {
        ((position.x - tx).powi(2) + (position.z - tz).powi(2)).sqrt()
    });
    check(
        tracker,
        edge > 0.0 && edge < centre,
        &format!(
            "the fireball's damage falls off away from the blast centre ({edge:.0} < {centre:.0}; \
             the other ork is {edge_distance:.0} from it)"
        ),
    );
    check(
        tracker,
        ally > 0.0,
        &format!("the fireball also hurts the player's own units ({ally:.0} damage)"),
    );
    let (x, z, before) = tracker.heartburn_ground;
    let after = units::ground_height(x, z);
    check(
        tracker,
        after < before - 1.0,
        &format!("the fireball leaves a crater (ground {before:.1} -> {after:.1})"),
    );

    let target = tracker.poultice_target;
    let (moved, away_dot) = spring::units_info::get_unit_position(
        target,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    )
    .map(|position| {
        let dx = position.x - tracker.poultice_x;
        let dz = position.z - tracker.poultice_z;
        let away_x = tracker.poultice_x - tracker.poultice_origin_x;
        let away_z = tracker.poultice_z - tracker.poultice_origin_z;
        ((dx * dx + dz * dz).sqrt(), dx * away_x + dz * away_z)
    })
    .unwrap_or((0.0, 0.0));
    check(
        tracker,
        target >= 0 && moved > 1.0 && away_dot > 1.0,
        &format!(
            "poultice throws an enemy unit away from the blood mage (moved {moved:.1}, away_dot {away_dot:.1})"
        ),
    );
    let airborne = spring::get_unit_physical_state(target)
        .map(|physical_state| (physical_state & PHYSICAL_STATE_FLYING) != 0)
        .unwrap_or(false);
    check(
        tracker,
        target >= 0 && airborne,
        "poultice target is airborne after the impulse",
    );
}

fn mage(world: &World) -> Option<(i32, spring::types::Float3)> {
    let mage = world.global.player.access().unit_id()?;
    Some((mage, units::position(mage)?))
}

const TRANSFUSION: i32 = 1;
const HEARTBURN: i32 = 2;
const ADRENALINE: i32 = 3;
const MIGRAINE: i32 = 4;
const POULTICE: i32 = 5;

fn send_spell(index: i32, target: spring::types::Float3) -> bool {
    springcabal::message::send_to_rules(
        &Command::Spell {
            index,
            x: target.x,
            y: target.y,
            z: target.z,
        }
        .message(),
    )
}

fn cast_taken(world: &World, sent: i32) -> bool {
    world.global.player.access().cast_start_frame() >= sent
}

pub(super) const VICTORY_ATTACK: i32 = 405;
pub(super) const VICTORY_ATTACK_CHECK: i32 = 420;

pub(super) fn attack_after_victory(world: &World) {
    let Some((_, position)) = mage(world) else {
        return;
    };
    let message = Encoder::new("attack")
        .field(position.x + 200.0)
        .field(position.z)
        .finish();
    let sent = springcabal::message::send_to_rules(&message);
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
