use core::cell::Cell;

use spring_addons::ui::Widget;

use crate::camera;
use crate::state::Global;

/// The close third-person camera that follows the blood mage.
pub struct GuiCameraWidget {
    initialized: Cell<bool>,
    /// The unit the engine's camera tracker follows.
    tracked: Cell<i32>,
}

impl GuiCameraWidget {
    pub fn new() -> Self {
        Self {
            initialized: Cell::new(false),
            tracked: Cell::new(-1),
        }
    }
}

impl Widget<Global> for GuiCameraWidget {
    fn name(&self) -> &'static str {
        "GuiCamera"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        camera::apply(&self.initialized);
        let player_unit = ctx.global().status.access().player_unit;
        camera::track(player_unit, &self.tracked);
    }
}
