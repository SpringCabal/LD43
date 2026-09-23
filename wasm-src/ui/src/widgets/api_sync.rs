use spring_addons::ui::Widget;

use crate::state::Global;

const STATUS_PREFIX: &str = "TTLD_STATUS|";

/// Receives the rules' status messages.
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

    fn recv_from_synced(&self, ctx: &spring_addons::AddonContext<'_, Global>, message: &[u8]) {
        let Ok(text) = core::str::from_utf8(message) else {
            return;
        };
        if text.starts_with(STATUS_PREFIX) {
            ctx.global().status.access_mut().apply_message(text);
        }
    }
}
