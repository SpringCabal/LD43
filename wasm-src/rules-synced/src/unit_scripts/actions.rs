use alloc::rc::Rc;
use core::cell::RefCell;
use spring::ResultExt;

use spring::UnitId;
use spring::cmd;
use spring::cus::UnitCtx;

use crate::random::unit_float;

use super::TtldState;
use super::common::face_direction;

pub(super) fn issue_patrol_orders(spring: &spring::Spring<'_, spring::RulesSynced>, ctx: &UnitCtx) {
    crate::engine::check(
        spring,
        spring
            .unit_control()
            .give_order_to_unit(ctx.unit().0, cmd::REPEAT, &[1.0], 0, 0),
        "give_order_to_unit",
        ctx.unit(),
    );
    if let Ok(position) = spring
        .units_info()
        .get_unit_position(ctx.unit().0, Default::default())
    {
        crate::engine::check(
            spring,
            spring.unit_control().give_order_to_unit(
                ctx.unit().0,
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

pub(super) fn face_from_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    x: f32,
    z: f32,
) {
    if let Ok(position) = spring
        .units_info()
        .get_unit_position(unit.0, Default::default())
    {
        face_direction(spring, unit, x - position.x, z - position.z);
    }
}

pub(super) fn coward_block(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    ctx: &UnitCtx,
    target: UnitId,
) -> bool {
    if target.0 < 0 {
        return true;
    }
    let (Some(target_position), Ok(unit_position)) = (
        spring
            .units_info()
            .get_unit_position(target.0, Default::default())
            .ok(),
        spring
            .units_info()
            .get_unit_position(ctx.unit().0, Default::default()),
    ) else {
        return true;
    };
    let dx = target_position.x - unit_position.x;
    let dz = target_position.z - unit_position.z;
    let cx = unit_position.x - dx + unit_float(spring) * 50.0 - 25.0;
    let cz = unit_position.z - dz + unit_float(spring) * 50.0 - 25.0;
    let cy = spring
        .terrain()
        .get_ground_height(cx, cz)
        .unwrap_or(unit_position.y);
    crate::engine::check(
        spring,
        spring
            .unit_control()
            .give_order_to_unit(ctx.unit().0, cmd::MOVE, &[cx, cy, cz], 0, 0),
        "give_order_to_unit",
        ctx.unit(),
    );
    spring
        .unit_control()
        .set_unit_weapon_state(
            ctx.unit().0,
            0,
            "reloadFrame",
            spring.game().current_frame().unwrap_or(0) as f32 + 180.0,
        )
        .log_err(spring, "coward reload");
    true
}

pub(super) fn ork_toss(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    target: i32,
    state: &Rc<RefCell<TtldState>>,
) {
    let (radius, strength, deunorkable) = {
        let state = state.borrow();
        (state.toss_radius, state.toss_strength, state.deunorkable)
    };
    if target < 0 || radius <= 0.0 {
        return;
    }
    let Some(target_position) = spring
        .units_info()
        .get_unit_position(target, Default::default())
        .ok()
    else {
        return;
    };
    let Some(unit_position) = spring
        .units_info()
        .get_unit_position(unit.0, Default::default())
        .ok()
    else {
        return;
    };
    let nearby = spring
        .units_query()
        .get_units_in_cylinder(target_position.x, target_position.z, radius, -1)
        .unwrap_or_default();
    for other in nearby {
        if other == unit.0 {
            continue;
        }
        let marked = spring
            .rules_params()
            .get_unit_rules_param_f32(other, "unorkable")
            .unwrap_or(0.0)
            > 0.0;
        if !deunorkable && marked {
            continue;
        }
        let Some(other_position) = spring
            .units_info()
            .get_unit_position(other, Default::default())
            .ok()
        else {
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
            spring,
            spring.unit_control().add_unit_impulse(
                other,
                spring::Float3::new(impulse * dx / distance, impulse, impulse * dz / distance),
                0.0,
            ),
            "add_unit_impulse",
            other,
        );
    }
}
