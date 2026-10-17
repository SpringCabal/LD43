use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

mod input;
use crate::state::Global;

pub struct GuiInputWidget {
    held: input::Held,
}

impl Default for GuiInputWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiInputWidget {
    pub fn new() -> Self {
        Self {
            held: input::Held::new(input::RESEND_DISTANCE),
        }
    }
}

impl Widget<Global> for GuiInputWidget {
    fn name(&self) -> &'static str {
        "GuiInput"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        input::sample_cursor(spring, ctx.global());
        input::clear_selection(spring);
        input::hold_mouse(spring, ctx.global(), &self.held);
    }

    fn mouse_move(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
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
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        up: bool,
        value: f32,
    ) -> EventResult {
        input::mouse_wheel(up, value).into()
    }

    fn mouse_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        let spring = ctx.spring();
        input::mouse_press(spring, ctx.global(), &self.held, x, y, button).into()
    }

    fn mouse_release(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        _x: i32,
        _y: i32,
        button: i32,
    ) -> EventResult {
        let spring = _ctx.spring();
        input::mouse_release(spring, &self.held, button).into()
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        let spring = ctx.spring();
        input::key_press(spring, ctx.global(), event).into()
    }
}
