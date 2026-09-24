use spring::consts::SQUARE_SIZE_F as SQUARE;

use crate::defs::defs;
use crate::units;
use spring::ResultExt;

const IMPASSIBLE_TERRAIN: i32 = 137;
const PLAYER_FUDGE: f32 = 16.0;

pub fn initialize() {
    spring::set_terrain_type_data(IMPASSIBLE_TERRAIN, 0.0, 0.0, 0.0, 0.0, 0.0, false, "")
        .log_err("impassable terrain type");
    spring::set_terrain_type_data(IMPASSIBLE_TERRAIN + 1, 0.4, 0.0, 0.0, 0.0, 0.0, false, "")
        .log_err("impassable terrain edge type");
}

pub fn block(unit_id: i32, unit_def_id: i32) {
    apply(unit_id, unit_def_id, IMPASSIBLE_TERRAIN);
}

pub fn unblock(unit_id: i32, unit_def_id: i32) {
    apply(unit_id, unit_def_id, 0);
}

/// Only immobile units, which are the ones with a footprint, block terrain.
fn apply(unit_id: i32, unit_def_id: i32, value: i32) {
    let Some(footprint) = defs().info(unit_def_id).footprint else {
        return;
    };
    let Some(extents) = extents(unit_id, footprint) else {
        return;
    };
    set_square(extents, value);
}

fn extents(unit_id: i32, (x_size, z_size): (f32, f32)) -> Option<(f32, f32, f32, f32)> {
    let position = units::position(unit_id)?;
    let facing = spring::units_info::get_unit_build_facing(unit_id).ok()?;
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

fn set_square(extents: (f32, f32, f32, f32), value: i32) {
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
            spring::set_map_square_terrain_type(x as i32, z as i32, terrain)
                .log_err_once("terrain type of a map square");
            z += SQUARE;
        }
        x += SQUARE;
    }
}
