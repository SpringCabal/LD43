use spring_addons::ui::Widget;
use springcabal_control::camera::CameraFollow;

use crate::camera;
use crate::state::Global;

/// The close third-person camera that follows the blood mage.
pub struct GuiCameraWidget {
    camera: CameraFollow,
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
        }
    }
}

impl Widget<Global> for GuiCameraWidget {
    fn name(&self) -> &'static str {
        "GuiCamera"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        let player_unit = ctx.global().status.access().player_unit;
        self.camera
            .update((player_unit >= 0).then_some(player_unit), dt);
    }
}
