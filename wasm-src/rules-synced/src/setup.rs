//! One-time world setup.

use crate::{diagnostics, state, terrain, town};

pub fn take_setup_flag() -> bool {
    state::with_world(|world| {
        let needs_setup = !world.initialized;
        world.initialized = true;
        needs_setup
    })
    .unwrap_or(false)
}

pub fn setup() {
    reveal_map();
    terrain::initialize();
    let villagers = town::populate();
    let _ = state::with_world(|world| world.villagers = villagers);
    diagnostics::info("world setup complete");
}

fn reveal_map() {
    for ally_team in 0..3 {
        let _ = spring::team_control::set_global_los(ally_team, true);
    }
}
