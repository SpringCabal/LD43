use spring::ResultExt;

use spring_addons::AddonContext;
use spring_addons::register_gadgets;
use spring_addons::rules::{Gadget, GadgetHandler};
use ttld_shared::Command;

use crate::gadgets::terrain::behavior as terrain;
#[cfg(feature = "pose-probe")]
use crate::pose_probe::PoseProbeGadget;
use crate::state::{Global, Phase};
use crate::units;

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

#[derive(Default)]
struct SetupGadget;

impl Gadget<Global> for SetupGadget {
    fn name(&self) -> &'static str {
        "Setup"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let global = ctx.global();
        if global.phase(frame) != Phase::Loading {
            return;
        }
        units::destroy_all(spring);
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .run(|_spring| crate::with_global(Global::forget_round))
        });
        global.begin_round(frame);
        for ally_team in 0..ALLY_TEAMS {
            spring
                .team_control()
                .set_global_los(ally_team, true)
                .log_err(
                    spring,
                    &alloc::format!("global LOS for ally team {ally_team}"),
                );
        }
        terrain::initialize(spring);
        spring::log_info!(
            spring,
            "ttld-wasm",
            "round {} setup started",
            global.round()
        );
    }

    fn handle_lua_msg(
        &self,
        ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let spring = ctx.spring();
        let command = core::str::from_utf8(data).ok().and_then(Command::parse);
        if command == Some(Command::Restart) {
            spring::log_info!(spring, "ttld-wasm", "restart requested");
            ctx.global().restart();
        }
    }
}

const ALLY_TEAMS: i32 = 2;

pub fn setup(
    handler: &mut GadgetHandler<Global>,
    _spring: &spring::Spring<'_, spring::RulesSynced>,
) {
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
    register_gadgets!(handler, [UnitScriptsGadget]);
}
