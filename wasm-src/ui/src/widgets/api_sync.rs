use spring_addons::ui::Widget;
use springcabal::message::key_values;

use crate::state::Global;

const STATUS_PREFIX: &str = "TTLD_STATUS";

/// Receives the rules' status messages, and starts a new round on the HUD
/// when their round changes.
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
        let Some(fields) = key_values(text, STATUS_PREFIX) else {
            return;
        };
        let global = ctx.global();
        let new_round = {
            let mut status = global.status.access_mut();
            let previous = status.round;
            status.apply(fields);
            previous != 0 && status.round != previous
        };
        if new_round {
            crate::hud::start_round(global);
        }
    }
}
