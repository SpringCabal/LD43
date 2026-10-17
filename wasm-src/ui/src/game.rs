use ttld_shared::STATUS_PERIOD;

pub fn status_frame(spring: &spring::Spring<'_, spring::Ui>) -> i32 {
    let frame = spring.game().current_frame().unwrap_or(0);
    frame - frame % STATUS_PERIOD
}
