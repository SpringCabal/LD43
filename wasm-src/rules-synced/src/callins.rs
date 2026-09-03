use alloc::format;

use crate::combat;
use crate::diagnostics;
use crate::player;
use crate::state;
use crate::status;
use crate::terrain;
use crate::town;
use crate::units::PLAYER_TEAM;

pub fn game_frame(frame: i32) {
    let _ = state::with_world(|world| world.frame = frame);
    if take_setup_flag() {
        setup();
        return;
    }
    combat::status_update(frame);
    player::resolve_pending(frame);
    drive_player(frame);
    advance_story(frame);
    steer_orks();
    steer_villagers();
    check_victory();
    if frame % STATUS_PERIOD == 0 {
        let _ = state::with_world(|world| status::publish(world, frame));
    }
    #[cfg(feature = "headless-integration")]
    crate::integration::update(frame);
}

pub fn handle_lua_msg(_player_id: i32, _script: i32, _mode: i32, data: &[u8]) {
    let Some(command) = player::parse(data) else {
        return;
    };
    let _ = state::with_world(|world| {
        let frame = world.frame;
        world.player.accept(command, frame);
    });
}

pub fn unit_created(unit_id: i32, unit_def_id: i32, unit_team: i32, _builder_id: i32) {
    terrain::block(unit_id, unit_def_id);
    let _ = state::with_world(|world| world.player.adopt(unit_id, unit_def_id));
    if unit_team == PLAYER_TEAM {
        combat::add_effect(
            unit_id,
            combat::Effect::HalfSpeed,
            combat::Modifier {
                move_factor: 0.5,
                reload_factor: 1.0,
            },
        );
    }
    let registered = state::with_world(|world| world.orks.created(unit_id, unit_def_id, unit_team))
        .unwrap_or(false);
    if registered {
        combat::advance(unit_id);
    }
}

pub fn unit_destroyed(
    unit_id: i32,
    unit_def_id: i32,
    _unit_team: i32,
    _attacker_id: i32,
    _attacker_def_id: i32,
    _attacker_team: i32,
    _weapon_def_id: i32,
) {
    terrain::unblock(unit_id, unit_def_id);
    combat::forget(unit_id);
    combat::status_forget(unit_id);
    let _ = state::with_world(|world| {
        world.remove_villager(unit_id);
        world.orks.destroyed(unit_id);
        world.player.release(unit_id);
    });
}

fn take_setup_flag() -> bool {
    state::with_world(|world| {
        let needs_setup = !world.initialized;
        world.initialized = true;
        needs_setup
    })
    .unwrap_or(false)
}

fn setup() {
    reveal_map();
    terrain::initialize();
    let villagers = town::populate();
    let _ = state::with_world(|world| world.villagers = villagers);
    diagnostics::info("world setup complete");
}

fn reveal_map() {
    for ally_team in 0..3 {
        let _ = spring::team_control::set_global_los(ally_team, true);
    }
}

fn drive_player(frame: i32) {
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

fn advance_story(frame: i32) {
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

fn steer_orks() {
    let batch = state::with_world(|world| world.orks.next_batch()).unwrap_or_default();
    for unit_id in batch {
        combat::check_idle(unit_id);
    }
}

fn steer_villagers() {
    let batch = state::with_world(|world| {
        let count = world.villagers.len().div_ceil(VILLAGER_UPDATE_DIVISOR);
        world.next_villagers(count)
    })
    .unwrap_or_default();
    for villager in batch {
        town::update(villager);
    }
}

fn check_victory() {
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
const STATUS_PERIOD: i32 = 6;
