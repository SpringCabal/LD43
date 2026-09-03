use crate::units;

const IMPASSIBLE_TERRAIN: i32 = 137;
const PLAYER_FUDGE: f32 = 16.0;
const SQUARE: f32 = 8.0;
const FOOTPRINT_TO_WORLD: f32 = 4.0;

pub fn initialize() {
    let _ = spring::set_terrain_type_data(IMPASSIBLE_TERRAIN, 0.0, 0.0, 0.0, 0.0, 0.0, false, "");
    let _ =
        spring::set_terrain_type_data(IMPASSIBLE_TERRAIN + 1, 0.4, 0.0, 0.0, 0.0, 0.0, false, "");
}

pub fn block(unit_id: i32, unit_def_id: i32) {
    apply(unit_id, unit_def_id, IMPASSIBLE_TERRAIN);
}

pub fn unblock(unit_id: i32, unit_def_id: i32) {
    apply(unit_id, unit_def_id, 0);
}

fn apply(unit_id: i32, unit_def_id: i32, value: i32) {
    if !is_immobile(unit_def_id) {
        return;
    }
    let Some(extents) = extents(unit_id, unit_def_id) else {
        return;
    };
    set_square(extents, value);
}

fn is_immobile(unit_def_id: i32) -> bool {
    spring::get_unit_def_speed(unit_def_id)
        .map(|speed| speed <= 0.0)
        .unwrap_or(false)
}

fn extents(unit_id: i32, unit_def_id: i32) -> Option<(f32, f32, f32, f32)> {
    let position = units::position(unit_id)?;
    let facing = spring::units_info::get_unit_build_facing(unit_id).ok()?;
    let x_size = footprint(unit_def_id, "xsize")? * FOOTPRINT_TO_WORLD;
    let z_size = footprint(unit_def_id, "zsize")? * FOOTPRINT_TO_WORLD;
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

fn footprint(unit_def_id: i32, key: &str) -> Option<f32> {
    spring::get_unit_def_param_int(unit_def_id, key)
        .ok()
        .map(|size| size as f32)
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
            let _ = spring::set_map_square_terrain_type(x as i32, z as i32, terrain);
            z += SQUARE;
        }
        x += SQUARE;
    }
}
