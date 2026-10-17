use alloc::format;
use alloc::vec::Vec;

use spring::cmd;

use crate::state::GameEnd;
use crate::units::PLAYER_TEAM;

use super::*;

#[derive(Clone, Copy)]
pub(super) struct RoundStart {
    after_setup: i32,
    units: usize,
    villagers: i32,
    enemies: i32,
}

#[derive(Clone, Copy)]
pub(super) struct RoundEnd {
    game_end: GameEnd,
    orks_killed: i32,
}

pub(super) const RESTART_DEADLINE: i32 = 600;

pub(super) fn record_round_start(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    tracker.first_round = Some(RoundStart {
        after_setup: world.frame - world.global.setup_frame().unwrap_or(0),
        units: living_units(spring).len(),
        villagers: living_villagers(spring),
        enemies: living_enemies(spring),
    });
}

pub(super) fn record_round_end(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    tracker.won_round = Some(RoundEnd {
        game_end: world.global.game_end.get(),
        orks_killed: world.global.score.access().orks_killed,
    });
}

pub(super) fn restart_check_due(tracker: &Tracker, world: &World) -> bool {
    let (Some(first), Some(setup)) = (tracker.first_round, world.global.setup_frame()) else {
        return false;
    };
    world.global.round() == 2 && world.frame == setup + first.after_setup
}

pub(super) fn restart_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    world: &World,
) {
    let global = world.global;
    let (Some(first), Some(won)) = (tracker.first_round, tracker.won_round) else {
        check(
            spring,
            tracker,
            false,
            "the first round was recorded before the restart",
        );
        return;
    };
    let game_end = global.game_end.get();
    check(
        spring,
        tracker,
        won.game_end == GameEnd::Victory && game_end == GameEnd::Start,
        &format!(
            "restarting after victory starts round {} from the start of the story",
            global.round()
        ),
    );
    let (villagers_killed, allies_drained, orks_killed) = {
        let score = global.score.access();
        (
            score.villagers_killed,
            score.allies_drained,
            score.orks_killed,
        )
    };
    check(
        spring,
        tracker,
        won.orks_killed > 0 && villagers_killed == 0 && allies_drained == 0 && orks_killed == 0,
        &format!(
            "the counters start from zero ({villagers_killed}/{allies_drained}/{orks_killed}; \
             the won round killed {} orks)",
            won.orks_killed
        ),
    );
    let (units, villagers) = (living_units(spring).len(), living_villagers(spring));
    check(
        spring,
        tracker,
        units == first.units && villagers == first.villagers,
        &format!(
            "the map holds a fresh town and nothing of the old round \
             ({units} units, {villagers} villagers; round 1: {}, {})",
            first.units, first.villagers
        ),
    );
    let player = global.player.access().unit_id();
    let fresh_mage = player.is_some_and(|unit_id| {
        let at_home = units::position(spring, unit_id)
            .is_some_and(|p| (p.x - MAGE_X).abs() < 50.0 && (p.z - MAGE_Z).abs() < 50.0);
        let full_health = units::health(spring, unit_id).is_some_and(|h| h.current >= h.max);
        let ours = spring.units_info().get_unit_team(unit_id).ok() == Some(PLAYER_TEAM);
        at_home && full_health && ours
    });
    check(
        spring,
        tracker,
        fresh_mage,
        &format!("a new blood mage at full health is the player unit ({player:?})"),
    );
    let orks = global.orks.access().units().to_vec();
    let attacking = orks
        .iter()
        .filter(|unit_id| {
            spring
                .units_commands()
                .get_command_queue(**unit_id, 1)
                .ok()
                .and_then(|queue| queue.into_iter().next())
                .is_some_and(|command| command.cmd_id == cmd::FIGHT)
        })
        .count();
    let enemies = living_enemies(spring);
    check(
        spring,
        tracker,
        enemies == first.enemies && orks.len() as i32 == enemies && attacking == orks.len(),
        &format!(
            "the first wave comes again and attacks ({attacking} of {enemies} orks; round 1: {})",
            first.enemies
        ),
    );
}

const MAGE_X: f32 = 5580.0;
const MAGE_Z: f32 = 7100.0;

fn living_units(spring: &spring::Spring<'_, spring::RulesSynced>) -> Vec<i32> {
    units::all_units(spring)
        .into_iter()
        .filter(|unit| units::health(spring, *unit).is_some_and(|health| health.current > 0.0))
        .collect()
}

fn living_enemies(spring: &spring::Spring<'_, spring::RulesSynced>) -> i32 {
    living_units(spring)
        .into_iter()
        .filter(|unit| spring.units_info().get_unit_team(*unit).ok() == Some(ENEMY_TEAM))
        .count() as i32
}

fn living_villagers(spring: &spring::Spring<'_, spring::RulesSynced>) -> i32 {
    let villager_defs = [
        crate::units::VillagerKind::Swordsman,
        crate::units::VillagerKind::Crossbowman,
        crate::units::VillagerKind::Coward,
        crate::units::VillagerKind::Peasant,
    ]
    .map(|kind| crate::defs::defs(spring).id(kind.unit()));
    living_units(spring)
        .into_iter()
        .filter(|unit| {
            spring.units_info().get_unit_team(*unit).ok() == Some(PLAYER_TEAM)
                && villager_defs.contains(&units::def_of(spring, *unit))
        })
        .count() as i32
}
