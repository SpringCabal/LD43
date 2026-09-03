//! Headless gameplay assertions.
//!
//! Enabled with the `headless-integration` cargo feature and driven from
//! `game_frame`, so every check runs against the same synced code path the real
//! game uses.  The runner (`wasm-test/verify-ttld.sh`) greps the log for the
//! final verdict line.

use alloc::format;
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::combat;
use crate::state::World;
use crate::units::{self, ENEMY_TEAM, OrkKind};

const TAG: &str = "[TTLD-Rust-Test]";
const TOWN_X: f32 = 5650.0;
const TOWN_Z: f32 = 5720.0;

#[derive(Default)]
struct Tracker {
    passed: u32,
    failed: u32,
    done: bool,
    ork_distance: f32,
    stunned_unit: i32,
    stunned_base: f32,
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
    let orks = world.orks.all();
    let mut total = 0.0;
    let mut counted = 0.0;
    for unit_id in &orks {
        let Some(position) = units::position(*unit_id) else {
            continue;
        };
        let dx = position.x - TOWN_X;
        let dz = position.z - TOWN_Z;
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
        .filter_map(|kind| units::def_id(kind.def_name()))
        .collect()
}

fn enemy_unit_count() -> i32 {
    ork_def_ids()
        .into_iter()
        .filter_map(|def_id| spring::get_team_unit_def_count(ENEMY_TEAM, def_id).ok())
        .map(|count| count as i32)
        .sum()
}

/// Called from `game_frame` with the world state *unborrowed*: several checks
/// spawn or destroy units, which re-enters this module through the call-ins.
pub fn update(frame: i32) {
    let done = TRACKER.with_borrow(|tracker| tracker.done);
    if done {
        return;
    }
    match frame {
        10 => with_checks(setup_checks),
        40 => with_checks(ork_spawn_checks),
        60 => with_checks(player_attack_checks),
        120 => spell_checks(),
        200 => with_checks(stun_a_target),
        230 => with_checks(stun_landed_checks),
        300 => with_checks(ork_advance_checks),
        330 => with_checks(victory_checks),
        360 => TRACKER.with_borrow_mut(finish),
        _ => {}
    }
}

fn with_checks(function: impl FnOnce(&mut Tracker, &mut World)) {
    let _ = crate::state::with_world(|world| {
        TRACKER.with_borrow_mut(|tracker| function(tracker, world));
    });
}

fn setup_checks(tracker: &mut Tracker, world: &mut World) {
    check(tracker, world.initialized, "world setup ran");
    check(
        tracker,
        world.villagers.len() > 300,
        &format!("town is populated ({} villagers)", world.villagers.len()),
    );
    let player = world.player.unit_id().unwrap_or(-1);
    check(
        tracker,
        player >= 0 && spring::valid_unit_id(player).unwrap_or(false),
        "blood mage spawned and was adopted as the player unit",
    );
    check(
        tracker,
        units::def_id("orksmall").is_some() && units::def_id("house").is_some(),
        "unit defs resolve (weapon/unit def tables loaded cleanly)",
    );
    // Houses are drawn scaled, and the unsynced UI reads the factors straight
    // out of the def's custom params.
    let scales = units::def_id("house")
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
    check(
        tracker,
        spring::owned::units_weapons::get_unit_weapon_state(player, 1, "reloadTime")
            .map(|state| state.reload_time > 0.0)
            .unwrap_or(false),
        "weapon state is readable for the player unit",
    );
}

fn ork_spawn_checks(tracker: &mut Tracker, world: &mut World) {
    let registered = world.orks.all().len();
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
    let with_orders = world
        .orks
        .all()
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

fn player_attack_checks(tracker: &mut Tracker, world: &mut World) {
    let Some(unit_id) = world.player.unit_id() else {
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
    world.player.accept(
        crate::player::parse(message.as_bytes()).expect("attack message parses"),
        frame_of(world),
    );
    world.player.update(frame_of(world));
    // Mirrors what scripts/blood_mage.lua reads to decide whether to fire.
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

fn spell_checks() {
    let Some((unit_id, position, frame, before)) = crate::state::with_world(|world| {
        let unit_id = world.player.unit_id()?;
        let position = units::position(unit_id)?;
        Some((unit_id, position, world.frame, world.allies_drained))
    })
    .flatten() else {
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
    // through UnitDestroyed, so it has to happen with the world unborrowed.
    let outcome = crate::player::cast(
        unit_id,
        crate::player::Spell::Adrenaline,
        position.x,
        position.z,
        frame,
    );
    let drained = crate::state::with_world(|world| world.allies_drained).unwrap_or(before);
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
}

/// Stunning scales the weapon's reload; make sure the weapon state really
/// changes and that reading it back reports the live value.
fn stun_a_target(tracker: &mut Tracker, world: &mut World) {
    // The mage rather than an ork: orks die and their ids get recycled, which
    // would make the follow-up read meaningless.
    let Some(unit_id) = world.player.unit_id() else {
        check(
            tracker,
            false,
            "the player unit is available for the stun test",
        );
        return;
    };
    tracker.stunned_unit = unit_id;
    tracker.stunned_base = live_reload(unit_id);
    combat::stun(unit_id, 300, world.frame);
}

fn stun_landed_checks(tracker: &mut Tracker, _world: &mut World) {
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

fn ork_advance_checks(tracker: &mut Tracker, world: &mut World) {
    let now = ork_distance(world);
    check(
        tracker,
        now > 0.0 && now < tracker.ork_distance,
        &format!(
            "orks advanced on the town ({:.0} -> {:.0})",
            tracker.ork_distance, now
        ),
    );
    check(
        tracker,
        world.villagers_killed > 0 || world.orks.killed() > 0,
        &format!(
            "combat is resolving (villagers lost {}, orks killed {})",
            world.villagers_killed,
            world.orks.killed()
        ),
    );
}

fn victory_checks(tracker: &mut Tracker, world: &mut World) {
    // The boss only appears in the last wave; short-circuit the schedule so the
    // victory path is exercised without playing the whole match.
    world.story.force_victory_possible();
    let possible = world.story.victory_possible();
    let declared = world.victory.update(possible);
    check(tracker, possible, "story can report the win condition");
    check(
        tracker,
        declared && world.game_end_state() == "victory",
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

fn frame_of(world: &World) -> i32 {
    world.frame
}
