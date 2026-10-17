use alloc::format;

use crate::defs::{UnitKind, defs};
use crate::state::Phase;
use crate::units;

use super::*;

pub(super) fn setup_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    check(
        spring,
        tracker,
        world.global.phase(world.frame) != Phase::Loading,
        "world setup ran",
    );
    let villagers = villager_count(spring);
    check(
        spring,
        tracker,
        villagers > 300,
        &format!("town is populated ({villagers} villagers)"),
    );
    let player = world.global.player.access().unit_id().unwrap_or(-1);
    check(
        spring,
        tracker,
        player >= 0 && spring.units_query().valid_unit_id(player).unwrap_or(false),
        "blood mage spawned and was adopted as the player unit",
    );
    check(
        spring,
        tracker,
        defs(spring).id(UnitKind::OrkSmall).is_some() && defs(spring).id(UnitKind::House).is_some(),
        "unit defs resolve (weapon/unit def tables loaded cleanly)",
    );
    let scales = defs(spring)
        .id(UnitKind::House)
        .map(|house| {
            (
                spring
                    .unit_defs()
                    .get_unit_def_custom_param(house, "hscale"),
                spring
                    .unit_defs()
                    .get_unit_def_custom_param(house, "vscale"),
            )
        })
        .unwrap_or((Err(spring::ApiError::new(0)), Err(spring::ApiError::new(0))));
    check(
        spring,
        tracker,
        scales.0.is_ok() && scales.1.is_ok(),
        &format!(
            "unit def custom params readable (house hscale={:?} vscale={:?})",
            scales.0, scales.1
        ),
    );
    let leader_scales = defs(spring)
        .id(UnitKind::OrkBoss)
        .map(|leader| {
            (
                spring
                    .unit_defs()
                    .get_unit_def_custom_param(leader, "hscale"),
                spring
                    .unit_defs()
                    .get_unit_def_custom_param(leader, "vscale"),
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
        spring,
        tracker,
        leader_scale_ok,
        &format!(
            "ork leader keeps its enlarged draw scale (hscale={:?} vscale={:?})",
            leader_scales.0, leader_scales.1
        ),
    );
    let gaia_ally_team = spring
        .game()
        .get_gaia_team_id()
        .and_then(|team| spring.teams().get_team_ally_team_id(team))
        .unwrap_or(-1);
    let los = |ally_team| spring.game().get_global_los(ally_team).unwrap_or(-1);
    check(
        spring,
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
        spring,
        tracker,
        spring
            .units_weapons()
            .get_unit_weapon_state(player, 1, "reloadTime")
            .map(|state| state.reload_time > 0.0)
            .unwrap_or(false),
        "weapon state is readable for the player unit",
    );
}

pub(super) fn ork_spawn_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    let orks = world.global.orks.access().units().to_vec();
    let registered = orks.len();
    let alive = enemy_unit_count(spring);
    check(
        spring,
        tracker,
        alive > 0,
        &format!("first wave spawned orks ({alive} alive)"),
    );
    check(
        spring,
        tracker,
        registered as i32 == alive,
        &format!("every spawned ork is registered with the AI ({registered}/{alive})"),
    );
    let with_orders = orks
        .into_iter()
        .filter(|unit_id| {
            spring
                .units_commands()
                .get_command_queue(*unit_id, 1)
                .map(|commands| !commands.is_empty())
                .unwrap_or(false)
        })
        .count();
    check(
        spring,
        tracker,
        with_orders == registered && registered > 0,
        &format!("every ork holds a fight order ({with_orders}/{registered})"),
    );
    tracker.ork_distance = ork_distance(spring, world);
}

pub(super) fn player_attack_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    world: &World,
) {
    let Some(unit_id) = world.global.player.access().unit_id() else {
        world.track(|tracker| {
            check(
                spring,
                tracker,
                false,
                "player unit is available for the attack test",
            )
        });
        return;
    };
    let Some(position) = units::position(spring, unit_id) else {
        world.track(|tracker| check(spring, tracker, false, "player unit has a position"));
        return;
    };
    world.track(|tracker| {
        check(
            spring,
            tracker,
            crate::unit_scripts::fire_target(spring, unit_id) == Some(None),
            "mage is not firing before an attack order",
        )
    });
    let message = format!("attack|{}|{}", position.x + 200.0, position.z);
    command_player(
        spring,
        world,
        crate::gadgets::player::behavior::parse(message.as_bytes()).expect("attack message parses"),
    );
    world.then_track(move |spring, tracker, _| {
        let target = crate::unit_scripts::fire_target(spring, unit_id).flatten();
        let fire_tx = target.map_or(0.0, |(tx, _)| tx);
        check(
            spring,
            tracker,
            target.is_some(),
            "attack order gives the mage script its fire target",
        );
        check(
            spring,
            tracker,
            fire_tx > position.x,
            &format!("fire target points at the attacked ground (fireTx {fire_tx:.0})"),
        );
    });
}

pub(super) fn command_player(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    world: &World,
    command: crate::gadgets::player::behavior::PlayerCommand,
) {
    let mut player = world.global.player.access_mut();
    player.accept(spring, command, world.frame);
    player.update(spring, world.frame);
}

pub(super) fn live_reload(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> f32 {
    spring
        .units_weapons()
        .get_unit_weapon_state(unit_id, 1, "reloadTime")
        .map(|state| state.reload_time)
        .unwrap_or(0.0)
}

pub(super) fn ork_advance_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    let now = ork_distance(spring, world);
    check(
        spring,
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
        spring,
        tracker,
        villagers_killed > 0 || orks_killed > 0,
        &format!(
            "combat is resolving (villagers lost {villagers_killed}, orks killed {orks_killed})"
        ),
    );
}
