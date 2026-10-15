use ttld_shared::STATUS_PERIOD;

pub fn status_frame() -> i32 {
    let frame = spring::game::current_frame();
    frame - frame % STATUS_PERIOD
}
