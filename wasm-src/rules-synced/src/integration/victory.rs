use alloc::format;

use spring::cmd;

use crate::combat::{self, Story};
use crate::state::GameEnd;

use super::*;

/// Runs the production victory path: the boss only appears in the last wave,
/// so the schedule is short-circuited to it. Declaring victory sends every ork
/// fleeing, which gives orders, so nothing is borrowed across it.
pub(super) fn victory_checks(world: &World) {
    let global = world.global;
    global.game_end.set(GameEnd::VictoryPossible);
    let declared = combat::declare_if_won(global);
    let game_end = global.game_end.get();
    let batch_empty = global.orks.access_mut().next_batch().is_empty();
    let orks = global.orks.access().units().to_vec();
    let retreating = orks.iter().filter(|unit| retreats_in_map(**unit)).count();
    // The Loop repeats forever; once the town is saved it sends nothing.
    let loop_stops = Story::at_the_loop()
        .take_due_wave(world.frame, GameEnd::VictoryPossible)
        .is_some()
        && Story::at_the_loop()
            .take_due_wave(world.frame, game_end)
            .is_none();
    world.track(|tracker| {
        check(
            tracker,
            declared && game_end == GameEnd::Victory,
            "victory is declared once the boss is gone",
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

/// Whether the unit's first order is a move to a point on the map.
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

/// A fleeing ork's reload factor is 0, which stretches its reload to at least
/// this many seconds.
pub(super) const FLEEING_RELOAD: f32 = 100_000.0;
