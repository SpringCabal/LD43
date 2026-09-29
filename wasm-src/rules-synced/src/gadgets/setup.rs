use spring::ResultExt;
use spring::commands;
use spring_addons::AddonContext;
use spring_addons::register_gadgets;
use spring_addons::rules::{Gadget, GadgetHandler};
use ttld_shared::Command;

#[cfg(feature = "pose-probe")]
use crate::pose_probe::PoseProbeGadget;
use crate::state::{Global, Phase};
use crate::terrain;
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

/// Starts a round, on the first game frame and on the first frame after the
/// player asks for a restart: removes every unit, forgets the previous
/// round, opens the map to the two player ally teams and prepares the terrain types.
/// The town itself is built by the town gadget in the same frame; see
/// [`Phase`].
#[derive(Default)]
struct SetupGadget;

impl Gadget<Global> for SetupGadget {
    fn name(&self) -> &'static str {
        "Setup"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        if global.phase(frame) != Phase::Loading {
            return;
        }
        // The destroys run once this frame's gadgets have returned, before
        // the commands they queue: the town gadget builds the new town on the
        // squares the old houses unblock. Their UnitDestroyed counts dead orks
        // and villagers, so the round state is forgotten after that.
        units::destroy_all();
        commands::run(|| crate::with_global(Global::forget_round));
        global.begin_round(frame);
        for ally_team in 0..ALLY_TEAMS {
            spring::team_control::set_global_los(ally_team, true)
                .log_err(&alloc::format!("global LOS for ally team {ally_team}"));
        }
        terrain::initialize();
        spring::log_info!("ttld-wasm", "round {} setup started", global.round());
    }

    /// The UI's restart (the end screen's Restart button, Enter or R).
    /// `ui_endgame.lua` reloads the Lua rules, which runs every gadget's
    /// Initialize again: the town handler clears the map and rebuilds the
    /// town, the storyteller and the ork AI start over, and the counters
    /// return to zero. That command doesn't reload this module, so the
    /// round starts over here.
    fn handle_lua_msg(
        &self,
        ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let command = core::str::from_utf8(data).ok().and_then(Command::parse);
        if command == Some(Command::Restart) {
            spring::log_info!("ttld-wasm", "restart requested");
            ctx.global().restart();
        }
    }
}

/// `game_setup_modrules.lua` opens the map to ally teams 0 and 1 only; in
/// the menu's two-team layout ally team 2 is Gaia.
const ALLY_TEAMS: i32 = 2;

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
    register_gadgets!(handler, [UnitScriptsGadget]);
}
