use spring_addons::ui::Widget;

use crate::hud;
use crate::state::Global;

#[derive(Default)]
pub struct DbgE2eWidget {
    ticks: core::cell::Cell<u32>,
}

impl DbgE2eWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let tick = self.ticks.get();
        self.ticks.set(tick + 1);
        match tick {
            5 => {
                let _ = spring::send_commands("screenshot", "");
            }
            10 => hud::close_intro(),
            30 | 70 | 120 => {
                let _ = spring::send_commands("screenshot", "");
            }
            _ => {}
        }
    }
}
