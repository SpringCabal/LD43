mod animation;
mod gameplay;
mod restart;
mod spells;
mod stun;
mod victory;

use self::animation::animation_checks;
use self::gameplay::*;
use self::restart::*;
use self::spells::*;
pub use self::stun::Bite;
use self::stun::*;
use self::victory::*;

use alloc::rc::Rc;
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
const CHECK_COUNT: u32 = 57;
const PHYSICAL_STATE_FLYING: u32 = 1 << 7;

#[derive(Default)]
pub struct Tracker {
    damage_house: Option<i32>,
    animation_unit: Option<i32>,
    walking_ork: Option<i32>,
    ork_heights: Vec<f32>,
    walk_heights: Vec<f32>,
    fire_poses: Vec<Vec<f32>>,
    shots: u32,
    suite: Checks,
    ork_distance: f32,
    stun_ork: i32,
    bites: Vec<Bite>,
    spell_sent: bool,
    drained_before_adrenaline: i32,
    drained_before_transfusion: i32,
    migraine_targets: Vec<i32>,
    victory_attack_sent: bool,
    heartburn_target: i32,
    heartburn_edge: i32,
    heartburn_ally: i32,
    heartburn_ground: (f32, f32, f32),
    fireball_hits: Vec<(i32, f32)>,
    bosses: Vec<i32>,
    poultice_target: i32,
    poultice_x: f32,
    poultice_z: f32,
    poultice_origin_x: f32,
    poultice_origin_z: f32,
    first_round: Option<RoundStart>,
    won_round: Option<RoundEnd>,
}

struct Checks(Suite);

impl Default for Checks {
    fn default() -> Self {
        Self(Suite::new(TAG, TITLE, CHECK_COUNT))
    }
}

fn check(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &mut Tracker,
    condition: bool,
    message: &str,
) {
    tracker.suite.0.check(spring, condition, message);
}

fn ork_distance(spring: &spring::Spring<'_, spring::RulesSynced>, world: &World) -> f32 {
    let orks = world.global.orks.access().units().to_vec();
    let mut total = 0.0;
    let mut counted = 0.0;
    for unit_id in &orks {
        let Some(position) = units::position(spring, *unit_id) else {
            continue;
        };
        let dx = position.x - TOWN_CENTRE_X;
        let dz = position.z - TOWN_CENTRE_Z;
        total += (dx * dx + dz * dz).sqrt();
        counted += 1.0;
    }
    if counted == 0.0 { 0.0 } else { total / counted }
}

fn ork_def_ids(spring: &spring::Spring<'_, spring::RulesSynced>) -> Vec<i32> {
    [OrkKind::Small, OrkKind::Big, OrkKind::Boss]
        .into_iter()
        .filter_map(|kind| defs(spring).id(kind.unit()))
        .collect()
}

fn villager_count(spring: &spring::Spring<'_, spring::RulesSynced>) -> i32 {
    [
        VillagerKind::Swordsman,
        VillagerKind::Crossbowman,
        VillagerKind::Coward,
        VillagerKind::Peasant,
    ]
    .into_iter()
    .filter_map(|kind| defs(spring).id(kind.unit()))
    .filter_map(|def_id| {
        spring
            .units_query()
            .get_team_unit_def_count(PLAYER_TEAM, def_id)
            .ok()
    })
    .map(|count| count as i32)
    .sum()
}

fn enemy_unit_count(spring: &spring::Spring<'_, spring::RulesSynced>) -> i32 {
    ork_def_ids(spring)
        .into_iter()
        .filter_map(|def_id| {
            spring
                .units_query()
                .get_team_unit_def_count(ENEMY_TEAM, def_id)
                .ok()
        })
        .map(|count| count as i32)
        .sum()
}

struct World<'a> {
    tracker: &'a Rc<RefCell<Tracker>>,
    global: &'a Global,
    frame: i32,
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    tracker: &Rc<RefCell<Tracker>>,
    global: &Global,
    frame: i32,
) {
    let world = World {
        tracker,
        global,
        frame,
    };
    let done = world.track(|tracker| tracker.suite.0.is_finished());
    if done {
        return;
    }
    animation_checks(spring, &world);
    house_fire_checks(spring, &world);
    match frame {
        10 => {
            with_checks(spring, &world, setup_checks);
            with_checks(spring, &world, record_round_start);
        }
        40 => with_checks(spring, &world, ork_spawn_checks),
        60 => player_attack_checks(spring, &world),
        62 => command_player(
            spring,
            &world,
            crate::gadgets::player::behavior::PlayerCommand::Stop,
        ),
        ADRENALINE_CAST => cast_adrenaline(spring, &world),
        POULTICE_CAST => cast_poultice(spring, &world),
        HEARTBURN_CAST => cast_heartburn(spring, &world),
        HEARTBURN_ALLY => heartburn_ally(spring, &world),
        SPELLS_LANDED_CHECK => with_checks(spring, &world, spells_landed_checks),
        STUN_SETUP => stun_fixture(spring, &world),
        STUN_START => with_checks(spring, &world, stun_the_ork),
        STUN_PAUSED_CHECK => with_checks(spring, &world, stunned_checks),
        STUN_OVER_CHECK => with_checks(spring, &world, stun_over_checks),
        TRANSFUSION_CAST => cast_transfusion(spring, &world),
        TRANSFUSION_CHECK => {
            with_checks(spring, &world, transfusion_checks);
            cast_migraine(spring, &world);
        }
        MIGRAINE_CHECK => migraine_checks(spring, &world),
        300 => with_checks(spring, &world, ork_advance_checks),
        330 => final_wave_checks(spring, &world),
        331 => kill_boss_checks(spring, &world),
        400 => victory_checks(spring, &world),
        VICTORY_ATTACK => attack_after_victory(spring, &world),
        VICTORY_ATTACK_CHECK => with_checks(spring, &world, attack_after_victory_checks),
        429 => {
            with_checks(spring, &world, retreat_persists_checks);
            with_checks(spring, &world, record_round_end);
        }
        RESTART_DEADLINE => world.track(|tracker| {
            check(
                spring,
                tracker,
                false,
                "the restart started a new round in time",
            );
            finish(spring, tracker);
        }),
        _ => {}
    }
    if world.track(|tracker| restart_check_due(tracker, &world)) {
        with_checks(spring, &world, restart_checks);
        world.track(|tracker| finish(spring, tracker));
    }
}

pub fn fireball_hit(tracker: &mut Tracker, unit_id: i32, damage: f32) {
    tracker.fireball_hits.push((unit_id, damage));
}

pub fn bite(tracker: &mut Tracker, bite: Bite) {
    tracker.bites.push(bite);
}

impl World<'_> {
    fn track<R>(&self, function: impl FnOnce(&mut Tracker) -> R) -> R {
        function(&mut self.tracker.borrow_mut())
    }

    fn then_track(
        &self,
        function: impl FnOnce(&spring::Spring<'_, spring::RulesSynced>, &mut Tracker, &Global) + 'static,
    ) {
        let tracker = Rc::clone(self.tracker);
        crate::with_addon_context(|ctx| {
            ctx.commands().run(move |spring| {
                crate::with_global(|global| function(spring, &mut tracker.borrow_mut(), global));
            })
        });
    }
}

fn with_checks(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    world: &World,
    function: impl FnOnce(&spring::Spring<'_, spring::RulesSynced>, &mut Tracker, &World),
) {
    world.track(|tracker| function(spring, tracker, world));
}

fn finish(spring: &spring::Spring<'_, spring::RulesSynced>, tracker: &mut Tracker) {
    tracker.suite.0.finish(spring);
}
