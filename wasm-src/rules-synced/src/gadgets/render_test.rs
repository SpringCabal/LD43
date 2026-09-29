//! Deterministic render fixtures used by the visual verification run.
//!
//! This is compiled only for the `e2e` build.  It puts an uncommanded boss and
//! the non-boss large ork beside the player, so the screenshot test can compare
//! the pixels produced by the unit draw callback without waiting for the final
//! story wave or depending on moving enemy AI.

use core::cell::Cell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;
use ttld_shared::Command;

use crate::defs::UnitKind;
use crate::state::Global;
use crate::units::{self, PLAYER_TEAM};

const FIXTURE_MESSAGE: &str = "TTLD E2E leader render fixture";

/// Spawns the leader render fixture when the e2e UI asks for it.
#[derive(Default)]
pub struct RenderTestGadget {
    spawned: Cell<bool>,
}

impl Gadget<Global> for RenderTestGadget {
    fn name(&self) -> &'static str {
        "RenderTest"
    }

    fn handle_lua_msg(
        &self,
        ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let command = core::str::from_utf8(data).ok().and_then(Command::parse);
        if command == Some(Command::LeaderRenderFixture) && !self.spawned.replace(true) {
            spawn_leader_fixture(ctx.global());
        }
    }
}

fn spawn_leader_fixture(global: &Global) {
    let player = global.player.access().unit_id();
    let Some(origin) = player.and_then(units::position) else {
        let _ = spring::echo("TTLD E2E leader render fixture skipped: no player", "");
        return;
    };

    // Keep both units on the same depth band so perspective does not become
    // the reason one silhouette is larger than the other.  OrkBig is the
    // non-boss control: it uses the same large-ork model family, but has the
    // ordinary 1x draw scale. They are on the player team solely to keep the
    // fixture stationary; the draw path is the same for allied and enemy
    // units.
    let (x, z) = (origin.x, origin.z);
    units::spawn(UnitKind::OrkBoss, x - 220.0, z - 140.0, PLAYER_TEAM, 0).then(move |boss| {
        units::spawn(UnitKind::OrkBig, x + 220.0, z - 140.0, PLAYER_TEAM, 0).then(move |orkbig| {
            let _ = spring::echo(
                &alloc::format!(
                    "{FIXTURE_MESSAGE}: boss={boss:?} orkbig={orkbig:?} origin=({x:.0},{z:.0})"
                ),
                "",
            );
        });
    });
}
