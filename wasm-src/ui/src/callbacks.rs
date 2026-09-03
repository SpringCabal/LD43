use crate::healthbars;
use crate::highlight;
use crate::hud;
use crate::mapedge;

pub fn dispatch(callback_id: u32, user_data: u32) {
    healthbars::dispatch_callback(callback_id, user_data);
    mapedge::dispatch_callback(callback_id);
    highlight::dispatch_callback(callback_id);
    hud::dispatch_callback(callback_id);
}
