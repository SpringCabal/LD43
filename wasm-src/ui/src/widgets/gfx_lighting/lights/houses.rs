use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::PointLight;

#[derive(Default)]
pub struct HouseLights {
    house_def: Option<i32>,
    lights: BTreeMap<i32, Vec<PointLight>>,
    repeatable: bool,
}

impl HouseLights {
    pub fn new(repeatable: bool) -> Self {
        Self {
            repeatable,
            ..Self::default()
        }
    }

    pub fn init(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        self.house_def = spring.unit_defs().get_unit_def_id_by_name(HOUSE).ok();
        for unit_id in spring.units_query().get_all_units().unwrap_or_default() {
            if let Ok(def_id) = spring.units_info().get_unit_def_id(unit_id) {
                self.created(spring, unit_id, def_id);
            }
        }
    }

    pub fn created(&mut self, spring: &spring::Spring<'_, spring::Ui>, unit_id: i32, def_id: i32) {
        if self.house_def != Some(def_id) {
            return;
        }
        let Ok(position) = spring.units_info().get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) else {
            return;
        };
        if self.repeatable && self.lights.is_empty() {
            spring
                .unsynced_random()
                .seed(REPEATABLE_SEED)
                .expect("unsynced random");
        }
        let mut lights = Vec::new();
        for dx in [-HALF_SIZE, HALF_SIZE] {
            for dz in [-HALF_SIZE, HALF_SIZE] {
                if spring.unsynced_random().float().expect("unsynced random") <= LIGHT_CHANCE {
                    continue;
                }
                lights.push(PointLight {
                    position: [position.x + dx, position.y + HEIGHT, position.z + dz],
                    color: [
                        RED * COLOR_MULTIPLIER,
                        GREEN * COLOR_MULTIPLIER,
                        BLUE * COLOR_MULTIPLIER,
                    ],
                    radius: RADIUS,
                });
            }
        }
        self.lights.insert(unit_id, lights);
    }

    pub fn destroyed(&mut self, unit_id: i32) {
        self.lights.remove(&unit_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &PointLight> {
        self.lights.values().flatten()
    }
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
