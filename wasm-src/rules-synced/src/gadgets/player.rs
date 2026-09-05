use spring_addons::rules::Gadget;

use crate::gadgets::setup::frame_consumed;
use crate::orchestration;
use crate::player;
use crate::state::Global;

#[derive(Default)]
pub struct PlayerGadget;

impl PlayerGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for PlayerGadget {
    fn name(&self) -> &'static str {
        "Player"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if frame_consumed() {
            return;
        }
        player::resolve_pending(frame);
        orchestration::drive_player(frame);
    }
}
