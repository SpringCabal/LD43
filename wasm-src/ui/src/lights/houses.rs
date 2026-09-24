use alloc::vec::Vec;

use super::PointLight;

/// Lights some of the windows of every house, once there are houses, like the
/// Lua widget's `math.random() > 0.6` per window. `repeatable` picks the same
/// windows every run.
pub fn place(repeatable: bool) -> Option<Vec<PointLight>> {
    let house_def_id = spring::get_unit_def_id_by_name(HOUSE).ok()?;
    let houses = houses_of(house_def_id);
    if houses.is_empty() {
        return None;
    }
    if repeatable {
        spring::random::unsynced::seed(REPEATABLE_SEED);
    }
    let mut placed = 0;
    let mut deferred_lights = Vec::new();
    for unit_id in houses {
        let Ok(position) = spring::units_info::get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) else {
            continue;
        };
        for dx in [-HALF_SIZE, HALF_SIZE] {
            for dz in [-HALF_SIZE, HALF_SIZE] {
                if spring::random::unsynced::float() <= LIGHT_CHANCE {
                    continue;
                }
                deferred_lights.push(PointLight {
                    position: [position.x + dx, position.y + HEIGHT, position.z + dz],
                    color: [
                        RED * COLOR_MULTIPLIER,
                        GREEN * COLOR_MULTIPLIER,
                        BLUE * COLOR_MULTIPLIER,
                    ],
                    radius: RADIUS,
                });
                placed += 1;
            }
        }
    }
    spring::log_info!("ttld-ui", "{placed} house lights placed");
    Some(deferred_lights)
}

const HOUSE: &str = "house";
const HALF_SIZE: f32 = 80.0;
const HEIGHT: f32 = 50.0;
const RADIUS: f32 = 500.0;
const RED: f32 = 0.7;
const GREEN: f32 = 0.7;
const BLUE: f32 = 0.4;
const COLOR_MULTIPLIER: f32 = 2.0;
const LIGHT_CHANCE: f32 = 0.6;
const REPEATABLE_SEED: i32 = 1;

fn houses_of(house_def_id: i32) -> Vec<i32> {
    spring::get_all_units()
        .unwrap_or_default()
        .into_iter()
        .filter(|unit_id| {
            spring::units_info::get_unit_def_id(*unit_id)
                .map(|def_id| def_id == house_def_id)
                .unwrap_or(false)
        })
        .collect()
}
