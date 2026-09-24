use alloc::vec::Vec;

use spring::cmd;

use crate::defs::UnitKind;
use crate::units::{self, PLAYER_TEAM};

use super::*;

/// Frames of the animation checks: a crossbowman walks, stops, then fires at
/// the ground, and an ork of the first wave walks to the town. They exercise
/// the real movement and weapon callins, not direct script calls.
pub(super) const ANIMATION_SPAWN: i32 = 15;
pub(super) const ANIMATION_WALK: i32 = 20;
pub(super) const ANIMATION_STOP: i32 = 75;
pub(super) const ANIMATION_FIRE: i32 = 85;
pub(super) const ANIMATION_CHECK: i32 = 195;
pub(super) const ORK_PICKED: i32 = 40;

pub(super) fn animation_checks(world: &World) {
    let frame = world.frame;
    if frame == ORK_PICKED {
        let ork = world
            .global
            .orks
            .access()
            .units()
            .iter()
            .copied()
            .find(|unit| {
                spring::get_unit_velocity(*unit)
                    .map(|v| v.x * v.x + v.z * v.z > 0.01)
                    .unwrap_or(false)
            });
        world.track(|tracker| tracker.walking_ork = ork);
    }
    if (ORK_PICKED + 1..100).contains(&frame)
        && let Some(ork) = world.track(|tracker| tracker.walking_ork)
        && let Some(matrix) = piece_matrix(ork, "Torso")
    {
        world.track(|tracker| tracker.ork_heights.push(matrix[13]));
    }
    if frame == ANIMATION_SPAWN {
        let unit = units::create(UnitKind::Crossbowman, 5580.0, 7700.0, PLAYER_TEAM, 0);
        world.track(|tracker| tracker.animation_unit = unit);
    }
    let Some(unit) = world.track(|tracker| tracker.animation_unit) else {
        if frame == ANIMATION_CHECK {
            world.track(|tracker| check(tracker, false, "crossbow animation fixture spawned"));
        }
        return;
    };
    match frame {
        ANIMATION_WALK => {
            let (x, z) = (5980.0, 7700.0);
            let _ = spring::give_order_to_unit(
                unit,
                cmd::MOVE,
                &[x, units::ground_height(x, z), z],
                0,
                0,
            );
        }
        ANIMATION_STOP => {
            let _ = spring::give_order_to_unit(unit, cmd::STOP, &[], 0, 0);
        }
        ANIMATION_FIRE => {
            if let Some(position) = units::position(unit) {
                let (x, z) = (position.x, position.z + 350.0);
                let _ = spring::give_order_to_unit(
                    unit,
                    cmd::ATTACK,
                    &[x, units::ground_height(x, z), z],
                    0,
                    0,
                );
            }
        }
        _ => {}
    }
    let walking = (ANIMATION_WALK + 1..70).contains(&frame);
    if walking || (ANIMATION_FIRE + 1..190).contains(&frame) {
        record_crossbowman(world, unit, walking);
    }
    if frame == ANIMATION_CHECK {
        world.track(report_animation);
        let _ = spring::destroy_unit(
            unit,
            spring::effects_control::DestroyUnitOptions {
                selfd: false,
                recycle_id: false,
                reclaimed: true,
                attacker_id: -1,
            },
        );
    }
}

/// Records the crossbowman's torso while it walks, its weapon hand while it
/// fires, and its projectiles in flight.
pub(super) fn record_crossbowman(world: &World, unit: i32, walking: bool) {
    let piece = if walking { "Torso" } else { "Hand_Right" };
    let matrix = piece_matrix(unit, piece);
    let shots =
        spring::projectiles::get_all_projectiles(spring::projectiles::GetAllProjectilesOptions {
            exclude_weapon_projectiles: false,
            exclude_piece_projectiles: true,
        })
        .unwrap_or_default()
        .into_iter()
        .filter(|projectile| {
            spring::projectiles::get_projectile_owner_id(*projectile).ok() == Some(unit)
        })
        .count() as u32;
    world.track(|tracker| {
        tracker.shots += shots;
        match (matrix, walking) {
            (Some(matrix), true) => tracker.walk_heights.push(matrix[13]),
            (Some(matrix), false) => tracker.fire_poses.push(matrix),
            (None, _) => {}
        }
    });
}

pub(super) fn report_animation(tracker: &mut Tracker) {
    let changes = |values: &[f32]| values.windows(2).any(|v| (v[0] - v[1]).abs() > 0.01);
    let ork_walking = changes(&tracker.ork_heights);
    check(
        tracker,
        ork_walking,
        "newly spawned advancing orks bob while walking",
    );
    let walking = changes(&tracker.walk_heights);
    check(
        tracker,
        walking,
        "walking changes the crossbowman's torso pose",
    );
    check(
        tracker,
        tracker.shots > 0,
        "crossbowman launches real weapon projectiles",
    );
    let firing = tracker
        .fire_poses
        .windows(2)
        .any(|v| v[0].iter().zip(&v[1]).any(|(a, b)| (a - b).abs() > 0.01));
    check(
        tracker,
        firing,
        "firing animates the crossbowman's weapon hand",
    );
}

pub(super) fn piece_matrix(unit: i32, name: &str) -> Option<Vec<f32>> {
    let map = spring::get_unit_piece_map(unit).ok()?;
    let piece = map.iter().find(|piece| piece.name == name)?;
    spring::owned::units_pieces::get_unit_piece_matrix(unit, piece.piece_num)
        .ok()
        .map(|matrix| matrix.m)
}
