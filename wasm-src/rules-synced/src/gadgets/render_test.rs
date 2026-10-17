use core::cell::Cell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;
use ttld_shared::Command;

use crate::defs::UnitKind;
use crate::state::Global;
use crate::units::{self, PLAYER_TEAM};

const FIXTURE_MESSAGE: &str = "TTLD E2E leader render fixture";

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
        let spring = ctx.spring();
        let command = core::str::from_utf8(data).ok().and_then(Command::parse);
        if command == Some(Command::LeaderRenderFixture) && !self.spawned.replace(true) {
            spawn_leader_fixture(spring, ctx.global());
        }
    }
}

fn spawn_leader_fixture(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let player = global.player.access().unit_id();
    let Some(origin) = player.and_then(|unit| units::position(spring, unit)) else {
        let _ = spring
            .messages()
            .echo("TTLD E2E leader render fixture skipped: no player", "");
        return;
    };

    let (x, z) = (origin.x, origin.z);
    units::spawn(
        spring,
        UnitKind::OrkBoss,
        x - 220.0,
        z - 140.0,
        PLAYER_TEAM,
        0,
    )
    .then(move |spring, boss| {
        units::spawn(
            spring,
            UnitKind::OrkBig,
            x + 220.0,
            z - 140.0,
            PLAYER_TEAM,
            0,
        )
        .then(move |spring, orkbig| {
            let _ = spring.messages().echo(
                &alloc::format!(
                    "{FIXTURE_MESSAGE}: boss={boss:?} orkbig={orkbig:?} origin=({x:.0},{z:.0})"
                ),
                "",
            );
        });
    });
}
