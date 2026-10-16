use spring::default_interface::{
    CLOCK, CONSOLE, END_GRAPH, PLAYER_INFO, RESOURCE_BAR, TOOLTIP, UNIT_ICONS,
};
use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};
use springcabal::music::MusicLoop;

use super::*;
use crate::state::Global;

const MUSIC_FILE: &str = "sounds/music.ogg";
const MUSIC_VOLUME: f32 = 0.20;

pub const HIDDEN_INTERFACE: u32 =
    CONSOLE | RESOURCE_BAR | TOOLTIP | CLOCK | PLAYER_INFO | UNIT_ICONS | END_GRAPH;

pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(handler, [springcabal::controls::GameControls::new()]);
    if springcabal::verification::world_only() {
        gallery::setup(handler);
        return;
    }
    register_widgets!(
        handler,
        [
            HideDefaultInterface::new(HIDDEN_INTERFACE & !CONSOLE),
            GuiInputWidget::new(),
            ApiSyncWidget::new(),
            GuiHudWidget::new(),
            GfxLightingWidget::new(cfg!(feature = "e2e")),
            GuiCameraWidget::new(),
            MusicLoop::new(MUSIC_FILE, MUSIC_VOLUME),
            GuiMinimapWidget::new(),
            GfxFireWidget::new(),
            GfxMapEdgeWidget::new(),
            GfxHighlightWidget::new(),
        ]
    );
    #[cfg(feature = "headless-integration")]
    register_widgets!(handler, [DbgRestartWidget::default()]);
    #[cfg(feature = "e2e")]
    register_widgets!(handler, [DbgE2eWidget::new()]);
}
