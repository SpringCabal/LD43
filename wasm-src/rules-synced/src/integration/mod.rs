//! Headless gameplay assertions.
//!
//! Enabled with the `headless-integration` cargo feature and driven from
//! `game_frame`, so every check runs against the same synced code path the real
//! game uses.  `verification/games/ttld.sh headless` greps the log for the
//! final verdict line.

mod animation;
mod gameplay;
mod victory;

use self::animation::animation_checks;
use self::gameplay::*;
use self::victory::*;

use alloc::vec::Vec;
use core::cell::RefCell;

use springcabal::check::Suite;

use crate::defs::defs;
use crate::state::Global;
use crate::units::{
    self, ENEMY_TEAM, OrkKind, PLAYER_TEAM, TOWN_CENTRE_X, TOWN_CENTRE_Z, VillagerKind,
};

const TAG: &str = "[TTLD-Rust-Test]";
const TITLE: &str = "TO THE LAST DROP INTEGRATION TESTS";
/// Every check below; fewer means some were skipped.
const CHECK_COUNT: u32 = 31;
/// CSolidObject::PSTATE_BIT_FLYING.
const PHYSICAL_STATE_FLYING: u32 = 1 << 7;

/// What the checks have seen so far; owned by the integration gadget.
#[derive(Default)]
pub struct Tracker {
    /// A crossbowman walked and then ordered to fire, for the animation checks.
    animation_unit: Option<i32>,
    /// An ork of the first wave that is on the move.
    walking_ork: Option<i32>,
    ork_heights: Vec<f32>,
    walk_heights: Vec<f32>,
    fire_poses: Vec<Vec<f32>>,
    shots: u32,
    suite: Checks,
    ork_distance: f32,
    stunned_unit: i32,
    stunned_base: f32,
    heartburn_target: i32,
    heartburn_health: f32,
    poultice_target: i32,
    poultice_x: f32,
    poultice_z: f32,
    poultice_origin_x: f32,
    poultice_origin_z: f32,
}

struct Checks(Suite);

impl Default for Checks {
    fn default() -> Self {
        Self(Suite::new(TAG, TITLE).require_at_least(CHECK_COUNT))
    }
}

fn check(tracker: &mut Tracker, condition: bool, message: &str) {
    tracker.suite.0.check(condition, message);
}

/// Mean distance of the living orks from the town centre they are ordered to
/// attack.  Used to prove the ork AI actually received and followed orders.
fn ork_distance(world: &World) -> f32 {
    let orks = world.global.orks.access().units().to_vec();
    let mut total = 0.0;
    let mut counted = 0.0;
    for unit_id in &orks {
        let Some(position) = units::position(*unit_id) else {
            continue;
        };
        let dx = position.x - TOWN_CENTRE_X;
        let dz = position.z - TOWN_CENTRE_Z;
        total += (dx * dx + dz * dz).sqrt();
        counted += 1.0;
    }
    if counted == 0.0 { 0.0 } else { total / counted }
}

fn rules_param(unit_id: i32, name: &str) -> Option<f32> {
    spring::get_unit_rules_param_f32(unit_id, name).ok()
}

fn ork_def_ids() -> Vec<i32> {
    [OrkKind::Small, OrkKind::Big, OrkKind::Boss]
        .into_iter()
        .filter_map(|kind| defs().id(kind.unit()))
        .collect()
}

fn villager_count() -> i32 {
    [
        VillagerKind::Swordsman,
        VillagerKind::Crossbowman,
        VillagerKind::Coward,
        VillagerKind::Peasant,
    ]
    .into_iter()
    .filter_map(|kind| defs().id(kind.unit()))
    .filter_map(|def_id| spring::get_team_unit_def_count(PLAYER_TEAM, def_id).ok())
    .map(|count| count as i32)
    .sum()
}

fn enemy_unit_count() -> i32 {
    ork_def_ids()
        .into_iter()
        .filter_map(|def_id| spring::get_team_unit_def_count(ENEMY_TEAM, def_id).ok())
        .map(|count| count as i32)
        .sum()
}

/// What the checks see: the gadgets' shared state and the current frame.
struct World<'a> {
    tracker: &'a RefCell<Tracker>,
    global: &'a Global,
    frame: i32,
}

/// Called from `game_frame` with no state borrowed: several checks spawn or
/// destroy units, which re-enters this module through the callins.
pub fn update(tracker: &RefCell<Tracker>, global: &Global, frame: i32) {
    let world = World {
        tracker,
        global,
        frame,
    };
    let done = world.track(|tracker| tracker.suite.0.is_finished());
    if done {
        return;
    }
    animation_checks(&world);
    match frame {
        10 => with_checks(&world, setup_checks),
        40 => with_checks(&world, ork_spawn_checks),
        60 => with_checks(&world, player_attack_checks),
        // The UI would send this when the mouse button comes up.
        62 => command_player(&world, crate::player::PlayerCommand::Stop),
        120 => spell_checks(&world),
        150 => with_checks(&world, heartburn_checks),
        200 => with_checks(&world, stun_a_target),
        230 => with_checks(&world, stun_landed_checks),
        300 => with_checks(&world, ork_advance_checks),
        330 => victory_checks(&world),
        359 => with_checks(&world, retreat_persists_checks),
        360 => world.track(finish),
        _ => {}
    }
}

impl World<'_> {
    fn track<R>(&self, function: impl FnOnce(&mut Tracker) -> R) -> R {
        function(&mut self.tracker.borrow_mut())
    }
}

fn with_checks(world: &World, function: impl FnOnce(&mut Tracker, &World)) {
    world.track(|tracker| function(tracker, world));
}

fn finish(tracker: &mut Tracker) {
    tracker.suite.0.finish();
}
