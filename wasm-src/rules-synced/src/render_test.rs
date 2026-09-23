//! Deterministic render fixtures used by the visual verification run.
//!
//! This is compiled only for the `e2e` build.  It puts an uncommanded boss and
//! the non-boss large ork beside the player, so the screenshot test can compare
//! the pixels produced by the unit draw callback without waiting for the final
//! story wave or depending on moving enemy AI.

use core::cell::Cell;

use crate::defs::UnitKind;
use crate::state;
use crate::units::{self, PLAYER_TEAM};

const FIXTURE_MESSAGE: &str = "TTLD E2E leader render fixture";

thread_local! {
    static SPAWNED: Cell<bool> = const { Cell::new(false) };
}

pub fn spawn_leader_fixture() {
    if SPAWNED.with(|spawned| spawned.replace(true)) {
        return;
    }

    let Some(origin) =
        state::with_world(|world| world.player.unit_id().and_then(units::position)).flatten()
    else {
        let _ = spring::echo("TTLD E2E leader render fixture skipped: no player", "");
        return;
    };

    // Keep both units on the same depth band so perspective does not become
    // the reason one silhouette is larger than the other.  OrkBig is the
    // non-boss control: it uses the same large-ork model family, but has the
    // ordinary 1x draw scale. They are on the player team solely to keep the
    // fixture stationary; the draw path is the same for allied and enemy
    // units.
    let boss = units::create(
        UnitKind::OrkBoss,
        origin.x - 220.0,
        origin.z - 140.0,
        PLAYER_TEAM,
        0,
    );
    let orkbig = units::create(
        UnitKind::OrkBig,
        origin.x + 220.0,
        origin.z - 140.0,
        PLAYER_TEAM,
        0,
    );
    let _ = spring::echo(
        &alloc::format!(
            "{FIXTURE_MESSAGE}: boss={:?} orkbig={:?} origin=({:.0},{:.0})",
            boss,
            orkbig,
            origin.x,
            origin.z
        ),
        "",
    );
}
