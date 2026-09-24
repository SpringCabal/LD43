use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

use crate::input;
use crate::state::Global;

/// Mouse and keyboard control of the blood mage. Registered before the other
/// game widgets: it samples the cursor for them at the start of every update.
#[derive(Default)]
pub struct GuiInputWidget {
    /// The command the held mouse button steers with.
    held: core::cell::Cell<Option<input::Held>>,
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
        input::clear_selection();
        input::hold_mouse(ctx.global(), &self.held);
    }

    fn mouse_move(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _x: i32,
        _y: i32,
        _dx: i32,
        _dy: i32,
        button: i32,
    ) -> EventResult {
        input::mouse_move(button).into()
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
        input::mouse_press(ctx.global(), &self.held, x, y, button).into()
    }

    fn mouse_release(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _x: i32,
        _y: i32,
        button: i32,
    ) -> EventResult {
        input::mouse_release(&self.held, button).into()
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        input::key_press(ctx.global(), event).into()
    }
}
