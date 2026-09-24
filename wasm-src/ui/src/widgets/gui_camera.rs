use core::cell::Cell;

use spring_addons::ui::Widget;
use springcabal_control::camera::CameraFollow;

use crate::camera;
use crate::state::Global;

/// The close third-person camera that follows the blood mage.
pub struct GuiCameraWidget {
    camera: CameraFollow,
    /// The rules' round the camera follows the blood mage of.
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
            camera: CameraFollow::new(camera::FOLLOW),
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
        // A restarted round starts from the start view again.
        if self.round.replace(round) != round && round > 1 {
            self.camera.reset();
        }
        self.camera
            .update((player_unit >= 0).then_some(player_unit), dt);
    }
}
