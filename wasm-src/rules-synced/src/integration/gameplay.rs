use alloc::format;

use crate::defs::{UnitKind, defs};
use crate::state::Phase;
use crate::units;

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
    let gaia_ally_team = spring::game::get_gaia_team_id(0)
        .and_then(spring::teams::get_team_ally_team_id)
        .unwrap_or(-1);
    let los = |ally_team| spring::game::get_global_los(ally_team).unwrap_or(-1);
    check(
        tracker,
        los(0) == 1 && los(1) == 1 && gaia_ally_team == 2 && los(gaia_ally_team) == 0,
        &format!(
            "global LOS for ally teams 0 and 1, not Gaia's ally team {gaia_ally_team} ({}, {}, {})",
            los(0),
            los(1),
            los(gaia_ally_team)
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

pub(super) fn player_attack_checks(world: &World) {
    let Some(unit_id) = world.global.player.access().unit_id() else {
        world.track(|tracker| {
            check(
                tracker,
                false,
                "player unit is available for the attack test",
            )
        });
        return;
    };
    let Some(position) = units::position(unit_id) else {
        world.track(|tracker| check(tracker, false, "player unit has a position"));
        return;
    };
    world.track(|tracker| {
        check(
            tracker,
            crate::unit_scripts::fire_target(unit_id) == Some(None),
            "mage is not firing before an attack order",
        )
    });
    let message = format!("attack|{}|{}", position.x + 200.0, position.z);
    command_player(
        world,
        crate::gadgets::player::behavior::parse(message.as_bytes()).expect("attack message parses"),
    );
    world.then_track(move |tracker, _| {
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
    });
}

pub(super) fn command_player(
    world: &World,
    command: crate::gadgets::player::behavior::PlayerCommand,
) {
    let mut player = world.global.player.access_mut();
    player.accept(command, world.frame);
    player.update(world.frame);
}

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
