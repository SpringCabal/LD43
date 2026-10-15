use core::cell::Cell;

use spring::Float3;
use spring_addons::ui::Widget;
use springcabal_control::camera::{Aim, CameraFollow, Follow, View};

use crate::state::Global;

pub struct GuiCameraWidget {
    camera: CameraFollow,
    round: Cell<u32>,
}

impl Default for GuiCameraWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiCameraWidget {
    pub fn new() -> Self {
        Self {
            camera: CameraFollow::new(FOLLOW),
            round: Cell::new(0),
        }
    }
}

impl Widget<Global> for GuiCameraWidget {
    fn name(&self) -> &'static str {
        "GuiCamera"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let (round, player_unit) = {
            let status = ctx.global().status.access();
            (status.round, status.player_unit)
        };
        if self.round.replace(round) != round && round > 1 {
            self.camera.reset();
        }
        self.camera
            .update((player_unit >= 0).then_some(player_unit), dt);
    }
}

const START_Y: f32 = 436.300_78;
const ROTATION_X: f32 = 2.5;
const START_VIEW: View = View::spring(1000.0, ROTATION_X)
    .pitch(-0.828_376_8, -0.560_171_2)
    .height(START_Y)
    .angle(ROTATION_X);
const START_POSITION: Float3 = Float3::new(5580.0, START_Y, 7100.0);

const FOLLOW: Follow = Follow::target(0.0)
    .start(START_VIEW, START_POSITION)
    .aim(Aim::MidPosition)
    .engine_tracker();
