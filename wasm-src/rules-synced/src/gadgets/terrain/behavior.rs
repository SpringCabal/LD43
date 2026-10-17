use spring::consts::SQUARE_SIZE_F as SQUARE;

use crate::defs::defs;
use crate::units;
use spring::ResultExt;

const IMPASSIBLE_TERRAIN: i32 = 137;
const PLAYER_FUDGE: f32 = 16.0;

pub fn initialize(spring: &spring::Spring<'_, spring::RulesSynced>) {
    spring
        .terrain_control()
        .set_terrain_type_data(IMPASSIBLE_TERRAIN, 0.0, 0.0, 0.0, 0.0, 0.0, false, "")
        .log_err(spring, "impassable terrain type");
    spring
        .terrain_control()
        .set_terrain_type_data(IMPASSIBLE_TERRAIN + 1, 0.4, 0.0, 0.0, 0.0, 0.0, false, "")
        .log_err(spring, "impassable terrain edge type");
}

pub fn block(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32, unit_def_id: i32) {
    apply(spring, unit_id, unit_def_id, IMPASSIBLE_TERRAIN);
}

pub fn unblock(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32, unit_def_id: i32) {
    apply(spring, unit_id, unit_def_id, 0);
}

fn apply(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    unit_def_id: i32,
    value: i32,
) {
    let Some(footprint) = defs(spring).info(spring, unit_def_id).footprint else {
        return;
    };
    let Some(extents) = extents(spring, unit_id, footprint) else {
        return;
    };
    set_square(spring, extents, value);
}

fn extents(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    (x_size, z_size): (f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    let position = units::position(spring, unit_id)?;
    let facing = spring.units_info().get_unit_build_facing(unit_id).ok()?;
    let (half_x, half_z) = if facing == 0 || facing == 2 {
        (x_size, z_size)
    } else {
        (z_size, x_size)
    };
    let center_x = snap(position.x, x_size);
    let center_z = snap(position.z, z_size);
    Some((
        center_x - half_x,
        center_z - half_z,
        center_x + half_x - SQUARE,
        center_z + half_z - SQUARE,
    ))
}

fn snap(value: f32, size: f32) -> f32 {
    if size % 16.0 == 0.0 {
        ((value + 8.0) / 16.0).floor() * 16.0
    } else {
        (value / 16.0).floor() * 16.0 + 8.0
    }
}

fn set_square(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    extents: (f32, f32, f32, f32),
    value: i32,
) {
    let (min_x, min_z, max_x, max_z) = extents;
    let mut x = min_x;
    while x <= max_x {
        let mut z = min_z;
        while z <= max_z {
            let edge = x < min_x + PLAYER_FUDGE
                || x > max_x - PLAYER_FUDGE
                || z < min_z + PLAYER_FUDGE
                || z > max_z - PLAYER_FUDGE;
            let terrain = if value == IMPASSIBLE_TERRAIN && edge {
                value + 1
            } else {
                value
            };
            spring
                .terrain_control()
                .set_map_square_terrain_type(x as i32, z as i32, terrain)
                .log_err_once(spring, "terrain type of a map square");
            z += SQUARE;
        }
        x += SQUARE;
    }
}
