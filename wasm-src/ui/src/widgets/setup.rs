use spring::default_interface::{
    CLOCK, CONSOLE, END_GRAPH, PLAYER_INFO, RESOURCE_BAR, TOOLTIP, UNIT_ICONS,
};
use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};
use springcabal::music::MusicLoop;
#[cfg(feature = "e2e")]
use springcabal::shots::{Clock, Screenshots};

use super::*;
use crate::state::Global;

const MUSIC_FILE: &str = "sounds/music.ogg";
const MUSIC_VOLUME: f32 = 0.20;

/// The minimap stays: the minimap widget draws the engine's.
pub const HIDDEN_INTERFACE: u32 =
    CONSOLE | RESOURCE_BAR | TOOLTIP | CLOCK | PLAYER_INFO | UNIT_ICONS | END_GRAPH;

/// Registers the widgets in the order they update and draw.
pub fn setup(handler: &mut WidgetHandler<Global>) {
    if gallery::active() {
        gallery::setup(handler);
        return;
    }
    register_widgets!(
        handler,
        [
            HideDefaultInterface::new(HIDDEN_INTERFACE),
            GuiInputWidget::new(),
            ApiSyncWidget::new(),
            GuiHudWidget::new(),
            GfxLightingWidget::new(),
            GuiCameraWidget::new(),
            MusicLoop::new(MUSIC_FILE, MUSIC_VOLUME),
            GuiMinimapWidget::new(),
            GfxFireWidget::new(),
            GfxMapEdgeWidget::new(),
            GfxHighlightWidget::new(),
        ]
    );
    #[cfg(feature = "e2e")]
    register_widgets!(
        handler,
        [
            DbgE2eWidget::new(),
            Screenshots::new(Clock::Update, &dbg_e2e::SCREENSHOTS),
        ]
    );
}
