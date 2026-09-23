use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

use crate::input;
use crate::state::Global;

/// Mouse and keyboard control of the blood mage. Registered first: it samples
/// the cursor for the other widgets at the start of every update.
#[derive(Default)]
pub struct GuiInputWidget {
    /// The game frame the held mouse button last steered on.
    held_frame: core::cell::Cell<i32>,
}

impl GuiInputWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiInputWidget {
    fn name(&self) -> &'static str {
        "GuiInput"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        input::sample_cursor(ctx.global());
        input::hold_mouse(ctx.global(), &self.held_frame);
    }

    fn mouse_move(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        _dx: i32,
        _dy: i32,
        _button: i32,
    ) -> EventResult {
        input::mouse_move(x, y).into()
    }

    fn mouse_wheel(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        up: bool,
        value: f32,
    ) -> EventResult {
        input::mouse_wheel(up, value).into()
    }

    fn mouse_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        input::mouse_press(ctx.global(), x, y, button).into()
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        input::key_press(ctx.global(), event.key_code, event.ctrl).into()
    }
}
