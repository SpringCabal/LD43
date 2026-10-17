pub mod mapedge;
use spring_addons::ui::Widget;

use self::mapedge::MapEdge;
use crate::state::Global;

#[derive(Default)]
pub struct GfxMapEdgeWidget {
    edge: MapEdge,
}

impl GfxMapEdgeWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GfxMapEdgeWidget {
    fn name(&self) -> &'static str {
        "GfxMapEdge"
    }

    fn draw_world_pre_unit(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        self.edge.draw(spring);
    }

    fn draw_world_refraction(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        self.edge.draw(spring);
    }
}
