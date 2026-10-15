use alloc::rc::Rc;

use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;
use springcabal::message::SendOnChange;

use ttld_shared::{STATUS_PERIOD, Status};

use crate::state::{Global, Phase};

#[derive(Default)]
pub struct StatusGadget {
    sent: Rc<SendOnChange<Status>>,
}

impl Gadget<Global> for StatusGadget {
    fn name(&self) -> &'static str {
        "Status"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        match global.phase(frame) {
            Phase::Loading => return,
            Phase::Setup => {
                self.sent.reset();
                return;
            }
            Phase::Running if frame % STATUS_PERIOD != 0 => return,
            Phase::Running => {}
        }
        let sent = Rc::clone(&self.sent);
        commands::run(move || {
            let status = crate::with_global(|global| read(global, frame));
            sent.send_if_changed(status, |status| {
                springcabal::message::send_to_ui(&status.message())
            });
        });
    }
}

fn read(global: &Global, frame: i32) -> Status {
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
    Status {
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

fn health(unit_id: i32) -> (i32, i32) {
    const UNKNOWN: (i32, i32) = (0, 1000);
    if unit_id < 0 {
        return UNKNOWN;
    }
    spring::units_info::get_unit_health(unit_id)
        .map(|health| (health.health.max(0.0) as i32, health.max_health as i32))
        .unwrap_or(UNKNOWN)
}
