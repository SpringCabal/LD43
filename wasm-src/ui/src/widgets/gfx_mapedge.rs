use spring_addons::ui::Widget;

use crate::mapedge;
use crate::state::Global;

/// The extended map edge, ported from `map_edge_extension.lua`.
#[derive(Default)]
pub struct GfxMapEdgeWidget;

impl GfxMapEdgeWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GfxMapEdgeWidget {
    fn name(&self) -> &'static str {
        "GfxMapEdge"
    }

    fn draw_world_pre_unit(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        mapedge::draw();
    }

    fn draw_world_refraction(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        mapedge::draw();
    }
}
