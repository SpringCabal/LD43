use spring_addons::AddonContext;
use spring_addons::register_gadgets;
use spring_addons::rules::{Gadget, GadgetHandler};

#[cfg(feature = "pose-probe")]
use crate::pose_probe::PoseProbeGadget;
use crate::state::{Global, Phase};
use crate::terrain;

use super::combat::CombatGadget;
#[cfg(feature = "headless-integration")]
use super::integration::IntegrationGadget;
use super::orks::OrksGadget;
use super::player::PlayerGadget;
#[cfg(feature = "e2e")]
use super::render_test::RenderTestGadget;
use super::status::StatusGadget;
use super::story::StoryGadget;
use super::terrain::TerrainGadget;
use super::town::TownGadget;
use super::unit_scripts::UnitScriptsGadget;
use super::victory::VictoryGadget;

/// Starts the world on the first game frame: opens the map to every ally team
/// and prepares the terrain types. The town itself is built by the town
/// gadget in the same frame; see [`Phase`].
#[derive(Default)]
struct SetupGadget;

impl Gadget<Global> for SetupGadget {
    fn name(&self) -> &'static str {
        "Setup"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().phase(frame) != Phase::Loading {
            return;
        }
        ctx.global().begin_setup(frame);
        for ally_team in 0..ALLY_TEAMS {
            let _ = spring::team_control::set_global_los(ally_team, true);
        }
        terrain::initialize();
        spring::log_info!("ttld-wasm", "world setup started");
    }
}

const ALLY_TEAMS: i32 = 3;

/// Registers the gadgets. Their order is the order they run in each frame,
/// which also fixes the order in which they draw from the synced RNG.
pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(
        handler,
        [
            SetupGadget,
            CombatGadget,
            TerrainGadget,
            PlayerGadget::default(),
            StoryGadget::default(),
            OrksGadget,
            TownGadget::default(),
            VictoryGadget,
            StatusGadget::default(),
        ]
    );
    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [IntegrationGadget::default()]);
    #[cfg(feature = "pose-probe")]
    register_gadgets!(handler, [PoseProbeGadget::new()]);
    #[cfg(feature = "e2e")]
    register_gadgets!(handler, [RenderTestGadget::default()]);
    register_gadgets!(handler, [UnitScriptsGadget::default()]);
}
