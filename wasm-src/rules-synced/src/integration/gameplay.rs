use alloc::format;

use crate::combat;
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
        rules_param(unit_id, "firing").unwrap_or(0.0) == 0.0,
        "mage is not firing before an attack order",
    );
    let message = format!("attack|{}|{}", position.x + 200.0, position.z);
    command_player(
        world,
        crate::player::parse(message.as_bytes()).expect("attack message parses"),
    );
    // The blood mage's unit script aims and fires from these params.
    let firing = rules_param(unit_id, "firing").unwrap_or(0.0);
    let fire_tx = rules_param(unit_id, "fireTx").unwrap_or(0.0);
    check(
        tracker,
        firing == 1.0,
        "attack order publishes the firing flag the mage script needs",
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

    let target = units::create(
        UnitKind::OrkSmall,
        position.x + 100.0,
        position.z,
        ENEMY_TEAM,
        0,
    );
    let target_health = target
        .and_then(|unit_id| spring::units_info::get_unit_health(unit_id).ok())
        .map(|health| health.health)
        .unwrap_or(0.0);
    let outcome = target.zip(crate::player::cast(
        global,
        unit_id,
        crate::player::Spell::Heartburn,
        position.x + 100.0,
        position.z,
        frame,
    ));
    world.track(|tracker| {
        if let Some((target, _)) = outcome {
            tracker.heartburn_target = target;
            tracker.heartburn_health = target_health;
        }
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
}

pub(super) fn heartburn_checks(tracker: &mut Tracker, _world: &World) {
    let target = tracker.heartburn_target;
    let health = spring::units_info::get_unit_health(target)
        .map(|health| health.health)
        .unwrap_or(0.0);
    check(
        tracker,
        target >= 0
            && (health < tracker.heartburn_health
                || !spring::valid_unit_id(target).unwrap_or(false)),
        "heartburn applies fireball damage at its destination",
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

/// Stunning scales the weapon's reload; make sure the weapon state really
/// changes and that reading it back reports the live value.
pub(super) fn stun_a_target(tracker: &mut Tracker, world: &World) {
    // The mage rather than an ork: orks die and their ids get recycled, which
    // would make the follow-up read meaningless.
    let Some(unit_id) = world.global.player.access().unit_id() else {
        check(
            tracker,
            false,
            "the player unit is available for the stun test",
        );
        return;
    };
    tracker.stunned_unit = unit_id;
    tracker.stunned_base = live_reload(unit_id);
    combat::stun(world.global, unit_id, 300, world.frame);
}

pub(super) fn stun_landed_checks(tracker: &mut Tracker, _world: &World) {
    let unit_id = tracker.stunned_unit;
    if unit_id <= 0 || !spring::valid_unit_id(unit_id).unwrap_or(false) {
        return;
    }
    let now = live_reload(unit_id);
    check(
        tracker,
        tracker.stunned_base > 0.0,
        &format!(
            "the base reload was readable ({:.2}s)",
            tracker.stunned_base
        ),
    );
    check(
        tracker,
        now > tracker.stunned_base,
        &format!(
            "a stun lengthens the weapon's live reload ({:.2}s -> {now:.0}s)",
            tracker.stunned_base
        ),
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
