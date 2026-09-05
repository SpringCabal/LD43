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

    fn recv_from_synced(&self, _ctx: &spring_addons::AddonContext<'_, Global>, message: &[u8]) {
        let Ok(text) = core::str::from_utf8(message) else {
            return;
        };
        if text.starts_with("TTLD_STATUS|") {
            hud::update(text);
        }
    }
}
