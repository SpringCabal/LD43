//! The per-frame steps the gadgets drive.

use alloc::format;

use crate::{combat, diagnostics, player, state, town};

pub fn drive_player(frame: i32) {
    let cast = state::with_world(|world| {
        let unit_id = world.player.unit_id()?;
        let (spell, x, z) = world.player.take_cast(frame)?;
        Some((unit_id, spell, x, z))
    })
    .flatten();
    if let Some((unit_id, spell, x, z)) = cast
        && let Some(outcome) = player::cast(unit_id, spell, x, z, frame)
    {
        let _ = state::with_world(|world| world.player.begin_cast(frame, outcome.cast_frames));
    }
    let _ = state::with_world(|world| world.player.update(frame));
}

pub fn advance_story(frame: i32) {
    let victory_declared = state::with_world(|world| world.victory.declared()).unwrap_or(false);
    if victory_declared {
        return;
    }
    // Spawn outside the world borrow: CreateUnit re-enters this module through
    // the UnitCreated call-in, which registers the ork with the AI.
    let wave: Option<&'static combat::Wave> =
        state::with_world(|world| world.story.take_due_wave(frame)).flatten();
    if let Some(wave) = wave {
        combat::spawn_wave(wave);
    }
}

pub fn steer_orks() {
    let batch = state::with_world(|world| world.orks.next_batch()).unwrap_or_default();
    for unit_id in batch {
        combat::check_idle(unit_id);
    }
}

pub fn steer_villagers() {
    let batch = state::with_world(|world| {
        let count = world.villagers.len().div_ceil(VILLAGER_UPDATE_DIVISOR);
        world.next_villagers(count)
    })
    .unwrap_or_default();
    for villager in batch {
        town::update(villager);
    }
}

pub fn check_victory() {
    let possible = state::with_world(|world| world.story.victory_possible()).unwrap_or(false);
    let won = state::with_world(|world| world.victory.update(possible)).unwrap_or(false);
    if !won {
        return;
    }
    diagnostics::info("victory");
    let orks = state::with_world(|world| {
        world.orks.start_fleeing();
        world.orks.all()
    })
    .unwrap_or_default();
    for unit_id in &orks {
        combat::flee(*unit_id);
    }
    diagnostics::info(&format!("{} orks fleeing", orks.len()));
}

const VILLAGER_UPDATE_DIVISOR: usize = 90;
