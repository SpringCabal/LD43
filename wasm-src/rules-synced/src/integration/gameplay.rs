use alloc::format;

use crate::defs::{UnitKind, defs};
use crate::state::Phase;
use crate::units::{self, ENEMY_TEAM};

use super::*;

pub(super) fn setup_checks(tracker: &mut Tracker, world: &World) {
    check(
        tracker,
        world.global.phase(world.frame) != Phase::Loading,
        "world setup ran",
    );
    let villagers = villager_count();
    check(
        tracker,
        villagers > 300,
        &format!("town is populated ({villagers} villagers)"),
    );
    let player = world.global.player.access().unit_id().unwrap_or(-1);
    check(
        tracker,
        player >= 0 && spring::valid_unit_id(player).unwrap_or(false),
        "blood mage spawned and was adopted as the player unit",
    );
    check(
        tracker,
        defs().id(UnitKind::OrkSmall).is_some() && defs().id(UnitKind::House).is_some(),
        "unit defs resolve (weapon/unit def tables loaded cleanly)",
    );
    // Houses are drawn scaled, and the unsynced UI reads the factors straight
    // out of the def's custom params.
    let scales = defs()
        .id(UnitKind::House)
        .map(|house| {
            (
                spring::get_unit_def_custom_param(house, "hscale"),
                spring::get_unit_def_custom_param(house, "vscale"),
            )
        })
        .unwrap_or((Err(spring::ApiError::new(0)), Err(spring::ApiError::new(0))));
    check(
        tracker,
        scales.0.is_ok() && scales.1.is_ok(),
        &format!(
            "unit def custom params readable (house hscale={:?} vscale={:?})",
            scales.0, scales.1
        ),
    );
    let leader_scales = defs()
        .id(UnitKind::OrkBoss)
        .map(|leader| {
            (
                spring::get_unit_def_custom_param(leader, "hscale"),
                spring::get_unit_def_custom_param(leader, "vscale"),
            )
        })
        .unwrap_or((Err(spring::ApiError::new(0)), Err(spring::ApiError::new(0))));
    let leader_scale_ok = leader_scales
        .0
        .as_ref()
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        == Some(2.5)
        && leader_scales
            .1
            .as_ref()
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            == Some(2.5);
    check(
        tracker,
        leader_scale_ok,
        &format!(
            "ork leader keeps its enlarged draw scale (hscale={:?} vscale={:?})",
            leader_scales.0, leader_scales.1
        ),
    );
    check(
        tracker,
        spring::owned::units_weapons::get_unit_weapon_state(player, 1, "reloadTime")
            .map(|state| state.reload_time > 0.0)
            .unwrap_or(false),
        "weapon state is readable for the player unit",
    );
}

pub(super) fn ork_spawn_checks(tracker: &mut Tracker, world: &World) {
    let orks = world.global.orks.access().units().to_vec();
    let registered = orks.len();
    let alive = enemy_unit_count();
    check(
        tracker,
        alive > 0,
        &format!("first wave spawned orks ({alive} alive)"),
    );
    // The regression this guards: CreateUnit re-enters the module through
    // UnitCreated, so spawning while the world was borrowed silently dropped
    // every ork and left them without orders for the whole match.
    check(
        tracker,
        registered as i32 == alive,
        &format!("every spawned ork is registered with the AI ({registered}/{alive})"),
    );
    let with_orders = orks
        .into_iter()
        .filter(|unit_id| {
            spring::units_commands::get_command_queue(*unit_id, 1)
                .map(|commands| !commands.is_empty())
                .unwrap_or(false)
        })
        .count();
    check(
        tracker,
        with_orders == registered && registered > 0,
        &format!("every ork holds a fight order ({with_orders}/{registered})"),
    );
    tracker.ork_distance = ork_distance(world);
}

pub(super) fn player_attack_checks(tracker: &mut Tracker, world: &World) {
    let Some(unit_id) = world.global.player.access().unit_id() else {
        check(
            tracker,
            false,
            "player unit is available for the attack test",
        );
        return;
    };
    let Some(position) = units::position(unit_id) else {
        check(tracker, false, "player unit has a position");
        return;
    };
    check(
        tracker,
        crate::unit_scripts::fire_target(unit_id) == Some(None),
        "mage is not firing before an attack order",
    );
    let message = format!("attack|{}|{}", position.x + 200.0, position.z);
    command_player(
        world,
        crate::player::parse(message.as_bytes()).expect("attack message parses"),
    );
    // The blood mage's unit script aims and fires at this target.
    let target = crate::unit_scripts::fire_target(unit_id).flatten();
    let fire_tx = target.map_or(0.0, |(tx, _)| tx);
    check(
        tracker,
        target.is_some(),
        "attack order gives the mage script its fire target",
    );
    check(
        tracker,
        fire_tx > position.x,
        &format!("fire target points at the attacked ground (fireTx {fire_tx:.0})"),
    );
}

/// Hands the player a command as the UI would, and issues the orders it gives.
pub(super) fn command_player(world: &World, command: crate::player::PlayerCommand) {
    let orders = {
        let mut player = world.global.player.access_mut();
        player.accept(command, world.frame);
        player.update(world.frame);
        player.take_orders()
    };
    orders.into_iter().for_each(crate::player::Order::issue);
}

pub(super) fn spell_checks(world: &World) {
    let global = world.global;
    let frame = world.frame;
    let before = global.score.access().allies_drained;
    let unit_id = global.player.access().unit_id();
    let Some((unit_id, position)) =
        unit_id.and_then(|unit_id| Some((unit_id, units::position(unit_id)?)))
    else {
        world.track(|tracker| {
            check(
                tracker,
                false,
                "player unit is available for the spell test",
            )
        });
        return;
    };
    // Casting drains (destroys) nearby villagers, which re-enters the module
    // through UnitDestroyed, so it has to happen with nothing borrowed.
    let outcome = crate::player::cast(
        global,
        unit_id,
        crate::player::Spell::Adrenaline,
        position.x,
        position.z,
        frame,
    );
    let drained = global.score.access().allies_drained;
    world.track(|tracker| {
        check(
            tracker,
            outcome.is_some(),
            "adrenaline cast is accepted and drains nearby allies",
        );
        check(
            tracker,
            drained > before,
            &format!("blood magic consumed villagers ({drained} drained)"),
        );
    });

    // Heartburn lands on the first ork; the second stands inside the blast
    // but away from its centre.
    let (tx, tz) = (position.x + 100.0, position.z);
    let target = units::create(UnitKind::OrkSmall, tx, tz, ENEMY_TEAM, 0);
    let edge = units::create(
        UnitKind::OrkSmall,
        tx,
        tz + HEARTBURN_EDGE_OFFSET,
        ENEMY_TEAM,
        0,
    );
    let ground = (tx, tz, units::ground_height(tx, tz));
    let cast = crate::player::cast(
        global,
        unit_id,
        crate::player::Spell::Heartburn,
        tx,
        tz,
        frame,
    );
    world.track(|tracker| {
        check(
            tracker,
            target.is_some() && edge.is_some() && cast.is_some(),
            "heartburn cast is accepted with its targets in place",
        );
        tracker.heartburn_target = target.unwrap_or(-1);
        tracker.heartburn_edge = edge.unwrap_or(-1);
        tracker.heartburn_ground = ground;
    });

    // Keep this target outside Heartburn's 250-unit damage radius. This is a
    // deterministic integration check for spell 5's delayed AddUnitImpulse,
    // rather than relying on a transient particle effect in a screenshot.
    let poultice_target = units::create(
        UnitKind::OrkSmall,
        position.x + 600.0,
        position.z,
        ENEMY_TEAM,
        0,
    );
    if let Some(target) = poultice_target
        && crate::player::cast(
            global,
            unit_id,
            crate::player::Spell::Poultice,
            position.x,
            position.z,
            frame,
        )
        .is_some()
        && let Some(before) = units::position(target)
    {
        world.track(|tracker| {
            tracker.poultice_target = target;
            tracker.poultice_x = before.x;
            tracker.poultice_z = before.z;
            tracker.poultice_origin_x = position.x;
            tracker.poultice_origin_z = position.z;
        });
    }

    // Created after the casts, so their blood magic can't drain it.
    let ally = units::create(
        UnitKind::Peasant,
        tx,
        tz - HEARTBURN_EDGE_OFFSET,
        crate::units::PLAYER_TEAM,
        0,
    );
    world.track(|tracker| tracker.heartburn_ally = ally.unwrap_or(-1));
}

/// How far from the fireball's target the other units in its blast stand.
/// Its area of effect is 500, so its radius is 250.
const HEARTBURN_EDGE_OFFSET: f32 = 170.0;

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
    check(
        tracker,
        edge > 0.0 && edge < centre,
        &format!(
            "the fireball's damage falls off away from the blast centre ({edge:.0} < {centre:.0})"
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

/// The weapon's current reload in seconds, straight from the engine.
pub(super) fn live_reload(unit_id: i32) -> f32 {
    spring::owned::units_weapons::get_unit_weapon_state(unit_id, 1, "reloadTime")
        .map(|state| state.reload_time)
        .unwrap_or(0.0)
}

pub(super) fn ork_advance_checks(tracker: &mut Tracker, world: &World) {
    let now = ork_distance(world);
    check(
        tracker,
        now > 0.0 && now < tracker.ork_distance,
        &format!(
            "orks advanced on the town ({:.0} -> {:.0})",
            tracker.ork_distance, now
        ),
    );
    let (villagers_killed, orks_killed) = {
        let score = world.global.score.access();
        (score.villagers_killed, score.orks_killed)
    };
    check(
        tracker,
        villagers_killed > 0 || orks_killed > 0,
        &format!(
            "combat is resolving (villagers lost {villagers_killed}, orks killed {orks_killed})"
        ),
    );
}
