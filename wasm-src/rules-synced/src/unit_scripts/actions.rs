//! Unit actions the script callbacks take.

use alloc::rc::Rc;
use core::cell::RefCell;

use spring::UnitId;
use spring::cmd;
use spring::cus::UnitCtx;

use crate::engine;
use crate::random::unit_float;

use super::TtldState;
use super::common::face_direction;

pub(super) fn issue_patrol_orders(ctx: &UnitCtx) {
    crate::engine::check(
        spring::give_order_to_unit(ctx.unit(), cmd::REPEAT, &[1.0], 0, 0),
        "give_order_to_unit",
        ctx.unit(),
    );
    if let Ok(position) = spring::get_unit_position(ctx.unit().0, false, false) {
        crate::engine::check(
            spring::give_order_to_unit(
                ctx.unit(),
                cmd::FIGHT,
                &[position.x, position.y, position.z],
                0,
                0,
            ),
            "give_order_to_unit",
            ctx.unit(),
        );
    }
}

pub(super) fn face_from_position(unit: UnitId, x: f32, z: f32) {
    if let Ok(position) = spring::get_unit_position(unit.0, false, false) {
        face_direction(unit, x - position.x, z - position.z);
    }
}

pub(super) fn coward_block(ctx: &UnitCtx, target: UnitId) -> bool {
    if target.0 < 0 {
        return true;
    }
    let (Some(target_position), Ok(unit_position)) = (
        spring::get_unit_position(target.0, false, false).ok(),
        spring::get_unit_position(ctx.unit().0, false, false),
    ) else {
        return true;
    };
    let dx = target_position.x - unit_position.x;
    let dz = target_position.z - unit_position.z;
    let cx = unit_position.x - dx + unit_float() * 50.0 - 25.0;
    let cz = unit_position.z - dz + unit_float() * 50.0 - 25.0;
    let cy = spring::get_ground_height(cx, cz).unwrap_or(unit_position.y);
    crate::engine::check(
        spring::give_order_to_unit(ctx.unit(), cmd::MOVE, &[cx, cy, cz], 0, 0),
        "give_order_to_unit",
        ctx.unit(),
    );
    let _ = spring::set_unit_weapon_state(
        ctx.unit().0,
        0,
        "reloadFrame",
        engine::frame() as f32 + 180.0,
    );
    true
}

pub(super) fn ork_toss(unit: UnitId, target: i32, state: &Rc<RefCell<TtldState>>) {
    let (radius, strength, deunorkable) = {
        let state = state.borrow();
        (state.toss_radius, state.toss_strength, state.deunorkable)
    };
    if target < 0 || radius <= 0.0 {
        return;
    }
    let Some(target_position) = spring::get_unit_position(target, false, false).ok() else {
        return;
    };
    let Some(unit_position) = spring::get_unit_position(unit.0, false, false).ok() else {
        return;
    };
    let nearby = spring::get_units_in_cylinder(target_position.x, target_position.z, radius, -1)
        .unwrap_or_default();
    for other in nearby {
        if other == unit.0 {
            continue;
        }
        let marked = spring::get_unit_rules_param_f32(other, "unorkable").unwrap_or(0.0) > 0.0;
        if !deunorkable && marked {
            continue;
        }
        let Some(other_position) = spring::get_unit_position(other, false, false).ok() else {
            continue;
        };
        let dx = other_position.x - unit_position.x;
        let dz = other_position.z - unit_position.z;
        let distance = (dx * dx + dz * dz).sqrt();
        if distance <= f32::EPSILON {
            continue;
        }
        let scale = if marked { 0.5 } else { 1.0 };
        let impulse = scale * strength * (1.0 - distance / (3.0 * radius));
        crate::engine::check(
            spring::add_unit_impulse(
                other,
                spring::Float3::new(impulse * dx / distance, impulse, impulse * dz / distance),
                0.0,
            ),
            "add_unit_impulse",
            other,
        );
    }
}
