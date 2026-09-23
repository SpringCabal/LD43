use spring_addons::ui::Widget;

use crate::hud;
use crate::state::Global;

#[derive(Default)]
pub struct ApiSyncWidget;

impl ApiSyncWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for ApiSyncWidget {
    fn name(&self) -> &'static str {
        "ApiSync"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let frame = crate::game::status_frame();
        let _ = crate::state::with_state(|ui| ui.game_frame = frame);
    }

    fn recv_from_synced(&self, _ctx: &spring_addons::AddonContext<'_, Global>, message: &[u8]) {
        let Ok(text) = core::str::from_utf8(message) else {
            return;
        };
        if text.starts_with("TTLD_STATUS|") {
            hud::update(text);
        }
    }
}
