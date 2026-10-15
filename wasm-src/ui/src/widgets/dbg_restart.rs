use core::cell::Cell;

use spring_addons::ui::Widget;

use crate::state::Global;

const RESTART_FRAME: i32 = 440;

#[derive(Default)]
pub struct DbgRestartWidget {
    sent: Cell<bool>,
}

impl Widget<Global> for DbgRestartWidget {
    fn name(&self) -> &'static str {
        "DbgRestart"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        if !self.sent.get() && spring::game::current_frame() >= RESTART_FRAME {
            self.sent.set(true);
            crate::widgets::gui_hud::hud::restart(ctx.global());
        }
    }
}
