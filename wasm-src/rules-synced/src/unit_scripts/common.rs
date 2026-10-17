use spring::ResultExt;
use spring::UnitId;
use spring::cus::{Angle, AngularSpeed, Axis, Piece, UnitCtx};

pub(super) const SIG_MOVE: spring::cus::SignalMask = spring::cus::SignalMask::new(1);
pub(super) const SIG_BUFF: spring::cus::SignalMask = spring::cus::SignalMask::new(2);
pub(super) const SIG_STUN: spring::cus::SignalMask = spring::cus::SignalMask::new(4);
pub(super) const SIG_ATTACK: spring::cus::SignalMask = spring::cus::SignalMask::new(8);
pub(super) const SIG_DEATH: spring::cus::SignalMask = spring::cus::SignalMask::new(16);

pub(super) fn turn(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if piece == Piece::INVALID {
        return;
    }
    if speed == 0.0 {
        ctx.turn_now(piece, axis, Angle::radians(target));
    } else {
        ctx.turn(
            piece,
            axis,
            Angle::radians(target),
            AngularSpeed::radians_per_second(speed),
        );
    }
}

pub(super) fn move_piece(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if piece == Piece::INVALID {
        return;
    }
    if speed == 0.0 {
        ctx.move_now(piece, axis, target);
    } else {
        ctx.move_piece(piece, axis, target, speed);
    }
}

pub(super) fn spin(ctx: &UnitCtx, piece: Piece, axis: Axis, speed: f32, acceleration: f32) {
    if piece != Piece::INVALID {
        ctx.spin(
            piece,
            axis,
            AngularSpeed::radians_per_second(speed),
            AngularSpeed::radians_per_second(acceleration),
        );
    }
}

pub(super) fn stop_spin(ctx: &UnitCtx, piece: Piece, axis: Axis, deceleration: f32) {
    if piece != Piece::INVALID {
        ctx.stop_spin(piece, axis, AngularSpeed::radians_per_second(deceleration));
    }
}

pub(super) fn face_direction(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    dx: f32,
    dz: f32,
) {
    if dx == 0.0 && dz == 0.0 {
        return;
    }
    let angle = dz.atan2(dx) - core::f32::consts::FRAC_PI_2;
    spring
        .unit_control()
        .set_unit_rotation(unit.0, spring::Float3::new(0.0, angle, 0.0))
        .log_err_once(spring, "facing a unit");
}

pub(super) fn face_target(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    target: i32,
) {
    if target < 0 {
        return;
    }
    let Some(unit_position) = spring
        .units_info()
        .get_unit_position(unit.0, Default::default())
        .ok()
    else {
        return;
    };
    let Some(target_position) = spring
        .units_info()
        .get_unit_position(target, Default::default())
        .ok()
    else {
        return;
    };
    face_direction(
        spring,
        unit,
        target_position.x - unit_position.x,
        target_position.z - unit_position.z,
    );
}

pub(super) fn play_unit_sound(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    name: &str,
    volume: f32,
) {
    let position = spring
        .units_info()
        .get_unit_position(unit.0, Default::default())
        .unwrap_or_default();
    spring
        .sound()
        .play_sound_file(name, volume, position, spring::Float3::default(), 0)
        .log_err_once(spring, name);
}

pub(super) fn set_blocking(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    blocking: bool,
) {
    spring
        .unit_control()
        .set_unit_blocking(
            unit.0,
            spring::unit_control::SetUnitBlockingOptions {
                blocking,
                solid_objects: blocking,
                projectiles: blocking,
                quad_map_rays: blocking,
                crushable: false,
                block_enemy_pushing: blocking,
                block_height_changes: blocking,
            },
        )
        .log_err(spring, &alloc::format!("blocking of unit {}", unit.0));
}

pub(super) fn collision_volume(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: UnitId,
    selection: bool,
) {
    let scales = spring::Float3::new(5.0, 5.0, 5.0);
    let offsets = spring::Float3::default();
    if selection {
        spring
            .unit_control()
            .set_unit_selection_volume_data(unit.0, scales, offsets, -1, 1, 0)
            .log_err(
                spring,
                &alloc::format!("selection volume of unit {}", unit.0),
            );
    } else {
        spring
            .unit_control()
            .set_unit_collision_volume_data(unit.0, scales, offsets, 0, 1, 0)
            .log_err(
                spring,
                &alloc::format!("collision volume of unit {}", unit.0),
            );
    }
}
