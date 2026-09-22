use alloc::vec::Vec;

use spring::UnitId;
use spring::cus::{Angle, AngularSpeed, Axis, Piece, PieceResolver, UnitCtx};

pub(super) const SIG_MOVE: spring::cus::SignalMask = spring::cus::SignalMask::new(1);
pub(super) const SIG_BUFF: spring::cus::SignalMask = spring::cus::SignalMask::new(2);
pub(super) const SIG_STUN: spring::cus::SignalMask = spring::cus::SignalMask::new(4);
pub(super) const SIG_ATTACK: spring::cus::SignalMask = spring::cus::SignalMask::new(8);
pub(super) const SIG_DEATH: spring::cus::SignalMask = spring::cus::SignalMask::new(16);

pub(super) struct EnginePieces {
    pub(super) map: Vec<spring::owned::units_pieces::PieceMapEntry>,
}

impl PieceResolver for EnginePieces {
    fn piece(&self, name: &str) -> Piece {
        self.map
            .iter()
            .find(|entry| entry.name == name)
            .map_or(Piece::INVALID, |entry| Piece(entry.piece_num - 1))
    }
}

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

pub(super) fn face_direction(unit: UnitId, dx: f32, dz: f32) {
    if dx == 0.0 && dz == 0.0 {
        return;
    }
    let angle = dz.atan2(dx) - core::f32::consts::FRAC_PI_2;
    let _ = spring::set_unit_rotation(unit.0, spring::types::Float3::new(0.0, angle, 0.0));
}

pub(super) fn face_target(unit: UnitId, target: i32) {
    if target < 0 {
        return;
    }
    let Some(unit_position) = spring::get_unit_position(unit.0, false, false).ok() else {
        return;
    };
    let Some(target_position) = spring::get_unit_position(target, false, false).ok() else {
        return;
    };
    face_direction(
        unit,
        target_position.x - unit_position.x,
        target_position.z - unit_position.z,
    );
}

pub(super) fn play_unit_sound(unit: UnitId, name: &str, volume: f32) {
    let position = spring::get_unit_position(unit.0, false, false).unwrap_or_default();
    let _ =
        spring::sound::play_sound_file(name, volume, position, spring::types::Float3::default(), 0);
}

pub(super) fn set_blocking(unit: UnitId, blocking: bool) {
    let _ = spring::set_unit_blocking(
        unit.0,
        spring::generated::owned::unit_control::SetUnitBlockingOptions {
            blocking,
            solid_objects: blocking,
            projectiles: blocking,
            quad_map_rays: blocking,
            crushable: false,
            block_enemy_pushing: blocking,
            block_height_changes: blocking,
        },
    );
}

pub(super) fn collision_volume(unit: UnitId, selection: bool) {
    let scales = spring::types::Float3::new(5.0, 5.0, 5.0);
    let offsets = spring::types::Float3::default();
    if selection {
        let _ = spring::set_unit_selection_volume_data(unit.0, scales, offsets, -1, 1, 0);
    } else {
        let _ = spring::set_unit_collision_volume_data(unit.0, scales, offsets, 0, 1, 0);
    }
}

pub(super) fn random_next(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *state
}

pub(super) fn random_unit(state: &mut u32) -> f32 {
    random_next(state) as f32 / u32::MAX as f32
}

pub(super) fn rules_value(value: f32) -> spring::generated::owned::rules_params::RulesParamValue {
    spring::generated::owned::rules_params::RulesParamValue {
        type_: spring::generated::owned::rules_params::RulesParamType::RulesparamTypeFloat,
        bool_value: false,
        float_value: value,
        string_value: alloc::string::String::new(),
    }
}
