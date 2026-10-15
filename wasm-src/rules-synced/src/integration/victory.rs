use alloc::format;

use spring::cmd;
use spring::commands;

use crate::defs::{UnitKind, defs};
use crate::gadgets::story::behavior::{self as story, Story};
use crate::gadgets::victory;
use crate::state::GameEnd;

use super::*;

pub(super) fn final_wave_checks(world: &World) {
    story::spawn_wave(Story::final_wave());
    world.then_track(|tracker, global| {
        let opened = victory::open_victory(global);
        let game_end = global.game_end.get();
        let bosses = boss_ids();
        check(
            tracker,
            opened && game_end == GameEnd::VictoryPossible && !bosses.is_empty(),
            &format!(
                "the final wave brings its boss and makes victory possible ({} boss)",
                bosses.len()
            ),
        );
        tracker.bosses = bosses;
    });
}

pub(super) fn kill_boss_checks(world: &World) {
    let game_end = world.global.game_end.get();
    let bosses = world.track(|tracker| {
        check(
            tracker,
            game_end == GameEnd::VictoryPossible,
            "no victory while the boss lives",
        );
        core::mem::take(&mut tracker.bosses)
    });
    for boss in bosses {
        let options = spring::effects_control::DestroyUnitOptions {
            selfd: false,
            recycle_id: false,
            reclaimed: false,
            attacker_id: -1,
        };
        commands::destroy_unit(boss, options).then(move |killed| {
            if let Err(error) = killed {
                spring::log_warning!("ttld-wasm", "boss {boss} not killed: {error:?}");
            }
        });
    }
}

pub(super) fn victory_checks(world: &World) {
    let global = world.global;
    let game_end = global.game_end.get();
    let batch_empty = global.orks.access_mut().next_batch().is_empty();
    let orks = global.orks.access().units().to_vec();
    let retreating = orks.iter().filter(|unit| retreats_in_map(**unit)).count();
    let loop_stops = Story::at_the_loop()
        .take_due_wave(world.frame, GameEnd::VictoryPossible)
        .is_some()
        && Story::at_the_loop()
            .take_due_wave(world.frame, game_end)
            .is_none();
    let bosses_left = boss_ids().len();
    world.track(|tracker| {
        check(
            tracker,
            bosses_left == 0 && game_end == GameEnd::Victory,
            &format!("victory is declared once the boss is gone ({bosses_left} left)"),
        );
        check(tracker, batch_empty, "victory disables attack AI batches");
        check(
            tracker,
            !orks.is_empty() && retreating == orks.len(),
            &format!(
                "surviving orks receive in-map retreat orders ({retreating}/{})",
                orks.len()
            ),
        );
        check(
            tracker,
            loop_stops,
            "The Loop does not spawn another wave after victory",
        );
    });
}

fn boss_ids() -> Vec<i32> {
    let Some(boss) = defs().id(UnitKind::OrkBoss) else {
        return Vec::new();
    };
    units::all_units()
        .into_iter()
        .filter(|unit_id| {
            units::def_of(*unit_id) == Some(boss)
                && spring::units_info::get_unit_team(*unit_id).ok() == Some(ENEMY_TEAM)
        })
        .collect()
}

pub(super) fn retreats_in_map(unit_id: i32) -> bool {
    let Ok(map) = spring::game::get_game_map_info(0) else {
        return false;
    };
    let (max_x, max_z) = (map.map_size_x as f32 - 1.0, map.map_size_z as f32 - 1.0);
    spring::units_commands::get_command_queue(unit_id, 1)
        .ok()
        .and_then(|queue| queue.into_iter().next())
        .is_some_and(|command| {
            command.cmd_id == cmd::MOVE
                && command.params.len() >= 3
                && (1.0..=max_x).contains(&command.params[0])
                && (1.0..=max_z).contains(&command.params[2])
        })
}

pub(super) fn retreat_persists_checks(tracker: &mut Tracker, world: &World) {
    let orks = world.global.orks.access().units().to_vec();
    let peaceful = orks.iter().all(|unit| {
        let no_attack = spring::units_commands::get_command_queue(*unit, u32::MAX)
            .map(|queue| {
                queue
                    .iter()
                    .all(|command| command.cmd_id != cmd::FIGHT && command.cmd_id != cmd::ATTACK)
            })
            .unwrap_or(false);
        no_attack && live_reload(*unit) >= FLEEING_RELOAD
    });
    check(
        tracker,
        peaceful,
        "retreating orks stay disarmed and receive no new attack orders",
    );
}

pub(super) const FLEEING_RELOAD: f32 = 100_000.0;
