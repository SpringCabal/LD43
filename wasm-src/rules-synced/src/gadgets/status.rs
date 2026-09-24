use alloc::string::String;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;
use springcabal::message::{Encoder, SendOnChange};

use crate::state::{GameEnd, Global, Phase};

/// Sends the game status to the UI, whenever it has changed, as a
/// `TTLD_STATUS|key=value|...` message.
#[derive(Default)]
pub struct StatusGadget {
    sent: SendOnChange<Status>,
}

/// How often, in frames, the status is checked for changes.
const STATUS_PERIOD: i32 = 6;

#[derive(Clone, Copy, PartialEq)]
struct Status {
    round: u32,
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
        match global.phase(frame) {
            Phase::Loading => return,
            // Every round starts with a status, even one that looks like the
            // last round's.
            Phase::Setup => {
                self.sent.reset();
                return;
            }
            Phase::Running if frame % STATUS_PERIOD != 0 => return,
            Phase::Running => {}
        }
        self.sent
            .send_if_changed(Status::read(global, frame), |status| {
                spring::send_to_unsynced(&status.message()).is_ok()
            });
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
            round: global.round(),
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
        Encoder::new("TTLD_STATUS")
            .kv("round", self.round)
            .kv("health", self.health)
            .kv("maxHealth", self.max_health)
            .kv("villagersKilled", self.villagers_killed)
            .kv("alliesDrained", self.allies_drained)
            .kv("orksKilled", self.orks_killed)
            .kv("gameEnd", self.game_end.name())
            .kv_flag("playerAlive", self.player_unit >= 0)
            .kv_flag("casting", self.casting)
            .kv("playerUnit", self.player_unit)
            .kv("castStart", self.cast_start)
            .kv("castAnimTime", self.cast_animation_frames)
            .finish()
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
