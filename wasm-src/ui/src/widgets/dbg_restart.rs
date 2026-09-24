use core::cell::Cell;

use spring_addons::ui::Widget;

use crate::state::Global;

/// The frame the headless integration test restarts the round, after the
/// rules' checks of the won round (`rules-synced/src/integration/mod.rs`).
const RESTART_FRAME: i32 = 440;

/// Restarts the round once in the headless integration test, the way the
/// end screen's Restart button does.
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
            crate::hud::restart(ctx.global());
        }
    }
}
