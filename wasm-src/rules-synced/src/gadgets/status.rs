use alloc::format;
use alloc::string::String;
use core::cell::Cell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::{GameEnd, Global};

/// Sends the game status to the UI, whenever it has changed, as a
/// `TTLD_STATUS|key=value|...` message.
#[derive(Default)]
pub struct StatusGadget {
    sent: Cell<Option<Status>>,
}

/// How often, in frames, the status is checked for changes.
const STATUS_PERIOD: i32 = 6;

#[derive(Clone, Copy, PartialEq)]
struct Status {
    player_unit: i32,
    health: i32,
    max_health: i32,
    casting: bool,
    cast_start: i32,
    cast_animation_frames: i32,
    villagers_killed: i32,
    allies_drained: i32,
    orks_killed: i32,
    game_end: GameEnd,
}

impl Gadget<Global> for StatusGadget {
    fn name(&self) -> &'static str {
        "Status"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        if !global.running(frame) || frame % STATUS_PERIOD != 0 {
            return;
        }
        let status = Status::read(global, frame);
        if self.sent.replace(Some(status)) != Some(status) {
            let _ = spring::send_to_unsynced(&status.message());
        }
    }
}

impl Status {
    fn read(global: &Global, frame: i32) -> Self {
        let (player_unit, casting, cast_start, cast_animation_frames) = {
            let player = global.player.access();
            (
                player.unit_id().unwrap_or(-1),
                player.casting(frame),
                player.cast_start_frame(),
                player.cast_animation_frames(),
            )
        };
        let (health, max_health) = health(player_unit);
        let score = global.score.access();
        Self {
            player_unit,
            health,
            max_health,
            casting,
            cast_start,
            cast_animation_frames,
            villagers_killed: score.villagers_killed,
            allies_drained: score.allies_drained,
            orks_killed: score.orks_killed,
            game_end: global.game_end.get(),
        }
    }

    fn message(&self) -> String {
        format!(
            "TTLD_STATUS|health={}|maxHealth={}|villagersKilled={}|alliesDrained={}|orksKilled={}|gameEnd={}|playerAlive={}|casting={}|playerUnit={}|castStart={}|castAnimTime={}",
            self.health,
            self.max_health,
            self.villagers_killed,
            self.allies_drained,
            self.orks_killed,
            self.game_end.name(),
            i32::from(self.player_unit >= 0),
            i32::from(self.casting),
            self.player_unit,
            self.cast_start,
            self.cast_animation_frames,
        )
    }
}

fn health(unit_id: i32) -> (i32, i32) {
    const UNKNOWN: (i32, i32) = (0, 1000);
    if unit_id < 0 {
        return UNKNOWN;
    }
    spring::units_info::get_unit_health(unit_id)
        .map(|health| (health.health.max(0.0) as i32, health.max_health as i32))
        .unwrap_or(UNKNOWN)
}
