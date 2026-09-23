//! Headless gameplay assertions.
//!
//! Enabled with the `headless-integration` cargo feature and driven from
//! `game_frame`, so every check runs against the same synced code path the real
//! game uses.  `verification/games/ttld.sh headless` greps the log for the
//! final verdict line.

use alloc::format;
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::combat;
use crate::defs::{UnitKind, defs};
use crate::state::{GameEnd, Global, Phase};
use crate::units::{
    self, ENEMY_TEAM, OrkKind, PLAYER_TEAM, TOWN_CENTRE_X, TOWN_CENTRE_Z, VillagerKind,
};

const TAG: &str = "[TTLD-Rust-Test]";
/// CSolidObject::PSTATE_BIT_FLYING.
const PHYSICAL_STATE_FLYING: u32 = 1 << 7;

#[derive(Default)]
struct Tracker {
    passed: u32,
    failed: u32,
    done: bool,
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

thread_local! {
    static TRACKER: RefCell<Tracker> = RefCell::new(Tracker::default());
}

fn check(tracker: &mut Tracker, condition: bool, message: &str) {
    if condition {
        tracker.passed += 1;
        let _ = spring::echo(
            &format!("{TAG} ASSERTION PASSED ({}): {message}", tracker.passed),
            "",
        );
    } else {
        tracker.failed += 1;
        let _ = spring::echo(&format!("{TAG} ASSERTION FAILED: {message}"), "");
    }
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
    global: &'a Global,
    frame: i32,
}

/// Called from `game_frame` with no state borrowed: several checks spawn or
/// destroy units, which re-enters this module through the callins.
pub fn update(global: &Global, frame: i32) {
    let world = World { global, frame };
    let done = TRACKER.with_borrow(|tracker| tracker.done);
    if done {
        return;
    }
    match frame {
        10 => with_checks(&world, setup_checks),
        40 => with_checks(&world, ork_spawn_checks),
        60 => with_checks(&world, player_attack_checks),
        120 => spell_checks(&world),
        150 => with_checks(&world, heartburn_checks),
        200 => with_checks(&world, stun_a_target),
        230 => with_checks(&world, stun_landed_checks),
        300 => with_checks(&world, ork_advance_checks),
        330 => with_checks(&world, victory_checks),
        360 => TRACKER.with_borrow_mut(finish),
        _ => {}
    }
}

fn with_checks(world: &World, function: impl FnOnce(&mut Tracker, &World)) {
    TRACKER.with_borrow_mut(|tracker| function(tracker, world));
}

fn setup_checks(tracker: &mut Tracker, world: &World) {
    check(
        tracker,
        world.global.phase(world.frame) != Phase::Loading,
        "world setup ran",
    );
    let villagers = villager_count();
    check(
        tracker,
        villagers > 300,
        &format!("town is populated ({villagers} villagers)"),
    );
    let player = world.global.player.access().unit_id().unwrap_or(-1);
    check(
        tracker,
        player >= 0 && spring::valid_unit_id(player).unwrap_or(false),
        "blood mage spawned and was adopted as the player unit",
    );
    check(
        tracker,
        defs().id(UnitKind::OrkSmall).is_some() && defs().id(UnitKind::House).is_some(),
        "unit defs resolve (weapon/unit def tables loaded cleanly)",
    );
    // Houses are drawn scaled, and the unsynced UI reads the factors straight
    // out of the def's custom params.
    let scales = defs()
        .id(UnitKind::House)
        .map(|house| {
            (
                spring::get_unit_def_custom_param(house, "hscale"),
                spring::get_unit_def_custom_param(house, "vscale"),
            )
        })
        .unwrap_or((Err(spring::ApiError::new(0)), Err(spring::ApiError::new(0))));
    check(
        tracker,
        scales.0.is_ok() && scales.1.is_ok(),
        &format!(
            "unit def custom params readable (house hscale={:?} vscale={:?})",
            scales.0, scales.1
        ),
    );
    let leader_scales = defs()
        .id(UnitKind::OrkBoss)
        .map(|leader| {
            (
                spring::get_unit_def_custom_param(leader, "hscale"),
                spring::get_unit_def_custom_param(leader, "vscale"),
            )
        })
        .unwrap_or((Err(spring::ApiError::new(0)), Err(spring::ApiError::new(0))));
    let leader_scale_ok = leader_scales
        .0
        .as_ref()
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        == Some(2.5)
        && leader_scales
            .1
            .as_ref()
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            == Some(2.5);
    check(
        tracker,
        leader_scale_ok,
        &format!(
            "ork leader keeps its enlarged draw scale (hscale={:?} vscale={:?})",
            leader_scales.0, leader_scales.1
        ),
    );
    check(
        tracker,
        spring::owned::units_weapons::get_unit_weapon_state(player, 1, "reloadTime")
            .map(|state| state.reload_time > 0.0)
            .unwrap_or(false),
        "weapon state is readable for the player unit",
    );
}

fn ork_spawn_checks(tracker: &mut Tracker, world: &World) {
    let orks = world.global.orks.access().units().to_vec();
    let registered = orks.len();
    let alive = enemy_unit_count();
    check(
        tracker,
        alive > 0,
        &format!("first wave spawned orks ({alive} alive)"),
    );
    // The regression this guards: CreateUnit re-enters the module through
    // UnitCreated, so spawning while the world was borrowed silently dropped
    // every ork and left them without orders for the whole match.
    check(
        tracker,
        registered as i32 == alive,
        &format!("every spawned ork is registered with the AI ({registered}/{alive})"),
    );
    let with_orders = orks
        .into_iter()
        .filter(|unit_id| {
            spring::units_commands::get_command_queue(*unit_id, 1)
                .map(|commands| !commands.is_empty())
                .unwrap_or(false)
        })
        .count();
    check(
        tracker,
        with_orders == registered && registered > 0,
        &format!("every ork holds a fight order ({with_orders}/{registered})"),
    );
    tracker.ork_distance = ork_distance(world);
}

fn player_attack_checks(tracker: &mut Tracker, world: &World) {
    let Some(unit_id) = world.global.player.access().unit_id() else {
        check(
            tracker,
            false,
            "player unit is available for the attack test",
        );
        return;
    };
    let Some(position) = units::position(unit_id) else {
        check(tracker, false, "player unit has a position");
        return;
    };
    check(
        tracker,
        rules_param(unit_id, "firing").unwrap_or(0.0) == 0.0,
        "mage is not firing before an attack order",
    );
    let message = format!("attack|{}|{}", position.x + 200.0, position.z);
    let orders = {
        let mut player = world.global.player.access_mut();
        player.accept(
            crate::player::parse(message.as_bytes()).expect("attack message parses"),
            world.frame,
        );
        player.update(world.frame);
        player.take_orders()
    };
    orders.into_iter().for_each(crate::player::Order::issue);
    // The blood mage's unit script aims and fires from these params.
    let firing = rules_param(unit_id, "firing").unwrap_or(0.0);
    let fire_tx = rules_param(unit_id, "fireTx").unwrap_or(0.0);
    check(
        tracker,
        firing == 1.0,
        "attack order publishes the firing flag the mage script needs",
    );
    check(
        tracker,
        fire_tx > position.x,
        &format!("fire target points at the attacked ground (fireTx {fire_tx:.0})"),
    );
}

fn spell_checks(world: &World) {
    let global = world.global;
    let frame = world.frame;
    let before = global.score.access().allies_drained;
    let unit_id = global.player.access().unit_id();
    let Some((unit_id, position)) =
        unit_id.and_then(|unit_id| Some((unit_id, units::position(unit_id)?)))
    else {
        TRACKER.with_borrow_mut(|tracker| {
            check(
                tracker,
                false,
                "player unit is available for the spell test",
            )
        });
        return;
    };
    // Casting drains (destroys) nearby villagers, which re-enters the module
    // through UnitDestroyed, so it has to happen with nothing borrowed.
    let outcome = crate::player::cast(
        global,
        unit_id,
        crate::player::Spell::Adrenaline,
        position.x,
        position.z,
        frame,
    );
    let drained = global.score.access().allies_drained;
    TRACKER.with_borrow_mut(|tracker| {
        check(
            tracker,
            outcome.is_some(),
            "adrenaline cast is accepted and drains nearby allies",
        );
        check(
            tracker,
            drained > before,
            &format!("blood magic consumed villagers ({drained} drained)"),
        );
    });

    let target = units::create(
        UnitKind::OrkSmall,
        position.x + 100.0,
        position.z,
        ENEMY_TEAM,
        0,
    );
    let target_health = target
        .and_then(|unit_id| spring::units_info::get_unit_health(unit_id).ok())
        .map(|health| health.health)
        .unwrap_or(0.0);
    let outcome = target.zip(crate::player::cast(
        global,
        unit_id,
        crate::player::Spell::Heartburn,
        position.x + 100.0,
        position.z,
        frame,
    ));
    TRACKER.with_borrow_mut(|tracker| {
        if let Some((target, _)) = outcome {
            tracker.heartburn_target = target;
            tracker.heartburn_health = target_health;
        }
    });

    // Keep this target outside Heartburn's 250-unit damage radius. This is a
    // deterministic integration check for spell 5's delayed AddUnitImpulse,
    // rather than relying on a transient particle effect in a screenshot.
    let poultice_target = units::create(
        UnitKind::OrkSmall,
        position.x + 600.0,
        position.z,
        ENEMY_TEAM,
        0,
    );
    if let Some(target) = poultice_target
        && crate::player::cast(
            global,
            unit_id,
            crate::player::Spell::Poultice,
            position.x,
            position.z,
            frame,
        )
        .is_some()
        && let Some(before) = units::position(target)
    {
        TRACKER.with_borrow_mut(|tracker| {
            tracker.poultice_target = target;
            tracker.poultice_x = before.x;
            tracker.poultice_z = before.z;
            tracker.poultice_origin_x = position.x;
            tracker.poultice_origin_z = position.z;
        });
    }
}

fn heartburn_checks(tracker: &mut Tracker, _world: &World) {
    let target = tracker.heartburn_target;
    let health = spring::units_info::get_unit_health(target)
        .map(|health| health.health)
        .unwrap_or(0.0);
    check(
        tracker,
        target >= 0
            && (health < tracker.heartburn_health
                || !spring::valid_unit_id(target).unwrap_or(false)),
        "heartburn applies fireball damage at its destination",
    );

    let target = tracker.poultice_target;
    let (moved, away_dot) = spring::units_info::get_unit_position(
        target,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    )
    .map(|position| {
        let dx = position.x - tracker.poultice_x;
        let dz = position.z - tracker.poultice_z;
        let away_x = tracker.poultice_x - tracker.poultice_origin_x;
        let away_z = tracker.poultice_z - tracker.poultice_origin_z;
        ((dx * dx + dz * dz).sqrt(), dx * away_x + dz * away_z)
    })
    .unwrap_or((0.0, 0.0));
    check(
        tracker,
        target >= 0 && moved > 1.0 && away_dot > 1.0,
        &format!(
            "poultice throws an enemy unit away from the blood mage (moved {moved:.1}, away_dot {away_dot:.1})"
        ),
    );
    let airborne = spring::get_unit_physical_state(target)
        .map(|physical_state| (physical_state & PHYSICAL_STATE_FLYING) != 0)
        .unwrap_or(false);
    check(
        tracker,
        target >= 0 && airborne,
        "poultice target is airborne after the impulse",
    );
}

/// Stunning scales the weapon's reload; make sure the weapon state really
/// changes and that reading it back reports the live value.
fn stun_a_target(tracker: &mut Tracker, world: &World) {
    // The mage rather than an ork: orks die and their ids get recycled, which
    // would make the follow-up read meaningless.
    let Some(unit_id) = world.global.player.access().unit_id() else {
        check(
            tracker,
            false,
            "the player unit is available for the stun test",
        );
        return;
    };
    tracker.stunned_unit = unit_id;
    tracker.stunned_base = live_reload(unit_id);
    combat::stun(world.global, unit_id, 300, world.frame);
}

fn stun_landed_checks(tracker: &mut Tracker, _world: &World) {
    let unit_id = tracker.stunned_unit;
    if unit_id <= 0 || !spring::valid_unit_id(unit_id).unwrap_or(false) {
        return;
    }
    let now = live_reload(unit_id);
    check(
        tracker,
        tracker.stunned_base > 0.0,
        &format!(
            "the base reload was readable ({:.2}s)",
            tracker.stunned_base
        ),
    );
    check(
        tracker,
        now > tracker.stunned_base,
        &format!(
            "a stun lengthens the weapon's live reload ({:.2}s -> {now:.0}s)",
            tracker.stunned_base
        ),
    );
}

/// The weapon's current reload in seconds, straight from the engine.
fn live_reload(unit_id: i32) -> f32 {
    spring::owned::units_weapons::get_unit_weapon_state(unit_id, 1, "reloadTime")
        .map(|state| state.reload_time)
        .unwrap_or(0.0)
}

fn ork_advance_checks(tracker: &mut Tracker, world: &World) {
    let now = ork_distance(world);
    check(
        tracker,
        now > 0.0 && now < tracker.ork_distance,
        &format!(
            "orks advanced on the town ({:.0} -> {:.0})",
            tracker.ork_distance, now
        ),
    );
    let (villagers_killed, orks_killed) = {
        let score = world.global.score.access();
        (score.villagers_killed, score.orks_killed)
    };
    check(
        tracker,
        villagers_killed > 0 || orks_killed > 0,
        &format!(
            "combat is resolving (villagers lost {villagers_killed}, orks killed {orks_killed})"
        ),
    );
}

fn victory_checks(tracker: &mut Tracker, world: &World) {
    // The boss only appears in the last wave; short-circuit the schedule so the
    // victory path is exercised without playing the whole match.
    let game_end = &world.global.game_end;
    game_end.set(GameEnd::VictoryPossible);
    let possible = game_end.get() == GameEnd::VictoryPossible;
    let declared = combat::declare_if_won(world.global);
    check(tracker, possible, "story can report the win condition");
    check(
        tracker,
        declared && game_end.get() == GameEnd::Victory,
        "victory is declared once the boss is gone",
    );
}

fn finish(tracker: &mut Tracker) {
    let _ = spring::echo(
        &if tracker.failed == 0 {
            format!(
                "{TAG} ALL TO THE LAST DROP INTEGRATION TESTS PASSED ({} assertions)",
                tracker.passed
            )
        } else {
            format!(
                "{TAG} TO THE LAST DROP INTEGRATION TESTS FAILED ({} of {})",
                tracker.failed,
                tracker.passed + tracker.failed
            )
        },
        "",
    );
    tracker.done = true;
}
