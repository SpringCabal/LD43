use spring_addons::ui::Widget;

use crate::audio;
use crate::state::Global;

#[derive(Default)]
pub struct SndMusicWidget;

impl SndMusicWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for SndMusicWidget {
    fn name(&self) -> &'static str {
        "SndMusic"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        audio::update();
    }
}
