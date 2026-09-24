use alloc::format;

use spring::cmd;

use crate::combat::{self, Story};
use crate::defs::{UnitKind, defs};
use crate::state::GameEnd;

use super::*;

/// Brings on the final wave the way the story does, since it only comes
/// after several minutes. Spawning and the gate open with nothing borrowed:
/// both re-enter the module through UnitCreated.
pub(super) fn final_wave_checks(world: &World) {
    let global = world.global;
    combat::spawn_wave(Story::final_wave());
    let opened = combat::open_victory(global);
    let game_end = global.game_end.get();
    let bosses = boss_ids();
    world.track(|tracker| {
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

/// A frame later the victory gadget has run: the living boss still stands in
/// the way. Then the boss is killed through the engine, which runs its Killed
/// script. It stays on the enemy team until its death sequence ends.
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
        let killed = spring::destroy_unit(
            boss,
            spring::effects_control::DestroyUnitOptions {
                selfd: false,
                recycle_id: false,
                reclaimed: false,
                attacker_id: -1,
            },
        );
        if let Err(error) = killed {
            spring::log_warning!("ttld-wasm", "boss {boss} not killed: {error:?}");
        }
    }
}

/// The victory gadget declared victory by itself once the boss's death
/// sequence (about 1.7 s) ended.
pub(super) fn victory_checks(world: &World) {
    let global = world.global;
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

/// The bosses on the enemy team.
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
