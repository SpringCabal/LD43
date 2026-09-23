use alloc::format;

use crate::engine::{set_game_number, set_game_string};
use crate::state::World;

pub fn publish(world: &World, frame: i32) {
    let player = world.player.unit_id().unwrap_or(-1);
    let (health, max_health) = health(player);
    set_game_number("villagersKilled", world.villagers_killed as f32);
    set_game_number("alliesDrained", world.allies_drained as f32);
    set_game_number("orksKilled", world.orks.killed() as f32);
    set_game_string("gameEnd", world.game_end_state());
    let message = format!(
        "TTLD_STATUS|frame={frame}|health={health}|maxHealth={max_health}|villagersKilled={}|alliesDrained={}|orksKilled={}|gameEnd={}|playerAlive={}|casting={}|playerUnit={player}|castStart={}|castAnimTime={}",
        world.villagers_killed,
        world.allies_drained,
        world.orks.killed(),
        world.game_end_state(),
        i32::from(player >= 0),
        i32::from(world.player.casting(frame)),
        world.player.cast_start_frame(),
        world.player.cast_animation_time(),
    );
    let _ = spring::send_to_unsynced(&message);
}

fn health(unit_id: i32) -> (i32, i32) {
    if unit_id < 0 {
        return (0, 1000);
    }
    spring::units_info::get_unit_health(unit_id)
        .map(|health| (health.health.max(0.0) as i32, health.max_health as i32))
        .unwrap_or((0, 1000))
}
