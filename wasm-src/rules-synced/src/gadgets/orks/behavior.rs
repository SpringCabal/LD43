use alloc::vec::Vec;

use spring::cmd;

use crate::defs::defs;
use crate::random;
use crate::state::Global;
use crate::units::{self, ENEMY_TEAM};

use crate::combat::{self as attributes, Effect, Modifier};

#[derive(Default)]
pub struct OrkAi {
    units: Vec<i32>,
    cursor: usize,
    fleeing: bool,
}

impl OrkAi {
    pub fn created(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        unit_id: i32,
        unit_def_id: i32,
        team: i32,
    ) -> bool {
        if team != ENEMY_TEAM || !is_ork(spring, unit_def_id) {
            return false;
        }
        self.units.push(unit_id);
        true
    }

    pub fn destroyed(&mut self, unit_id: i32) -> bool {
        let Some(index) = self.units.iter().position(|id| *id == unit_id) else {
            return false;
        };
        self.units.swap_remove(index);
        if self.cursor >= self.units.len() {
            self.cursor = 0;
        }
        true
    }

    pub fn next_batch(&mut self) -> Vec<i32> {
        if self.fleeing || self.units.is_empty() {
            return Vec::new();
        }
        let count = self.units.len().div_ceil(UPDATE_DIVISOR);
        let mut batch = Vec::with_capacity(count);
        for _ in 0..count {
            if self.cursor >= self.units.len() {
                self.cursor = 0;
            }
            batch.push(self.units[self.cursor]);
            self.cursor += 1;
        }
        batch
    }

    pub fn units(&self) -> &[i32] {
        &self.units
    }

    pub fn start_fleeing(&mut self) {
        self.fleeing = true;
    }
}

pub fn advance(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    let dx = AIM_X + random::unit_float(spring) * AIM_VARIANCE - AIM_VARIANCE / 2.0;
    let dz = AIM_Z + random::unit_float(spring) * AIM_VARIANCE - AIM_VARIANCE / 2.0;
    let y = units::ground_height(spring, dx, dz);
    crate::engine::check(
        spring,
        spring
            .unit_control()
            .give_order_to_unit(unit_id, cmd::FIGHT, &[dx, y, dz], 0, 0),
        "give_order_to_unit",
        unit_id,
    );
}

pub fn check_idle(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    if units::has_orders(spring, unit_id) {
        return;
    }
    let dx = AIM_X + random::unit_float(spring) * AIM_REACHED_VARIANCE - AIM_REACHED_VARIANCE / 2.0;
    let dz = AIM_Z + random::unit_float(spring) * AIM_REACHED_VARIANCE - AIM_REACHED_VARIANCE / 2.0;
    let y = units::ground_height(spring, dx, dz);
    order(spring, unit_id, cmd::FIGHT, &[dx, y, dz]);
}

pub fn flee(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, unit_id: i32) {
    let Some(position) = units::position(spring, unit_id) else {
        return;
    };
    let dx = position.x - AIM_X;
    let dz = position.z - AIM_Z;
    let distance = (dx * dx + dz * dz).sqrt().max(0.1);
    let Ok(map) = spring.game().get_game_map_info() else {
        return;
    };
    let cx = (position.x + FLEE_DISTANCE * dx / distance).clamp(1.0, map.map_size_x as f32 - 1.0);
    let cz = (position.z + FLEE_DISTANCE * dz / distance).clamp(1.0, map.map_size_z as f32 - 1.0);
    let cy = units::ground_height(spring, cx, cz);
    order(spring, unit_id, cmd::FIRE_STATE, &[0.0]);
    attributes::add_effect(
        spring,
        global,
        unit_id,
        Effect::Flee,
        Modifier {
            move_factor: 1.0,
            reload_factor: 0.0,
        },
    );
    order(spring, unit_id, cmd::MOVE, &[cx, cy, cz]);
}

const AIM_X: f32 = units::TOWN_CENTRE_X;
const AIM_Z: f32 = units::TOWN_CENTRE_Z;
const AIM_VARIANCE: f32 = 600.0;
const AIM_REACHED_VARIANCE: f32 = 2000.0;
const FLEE_DISTANCE: f32 = 4000.0;
const UPDATE_DIVISOR: usize = 80;

fn is_ork(spring: &spring::Spring<'_, spring::RulesSynced>, unit_def_id: i32) -> bool {
    defs(spring).is_ork(unit_def_id)
}

fn order(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    cmd_id: i32,
    params: &[f32],
) {
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .give_order_to_unit(unit_id, cmd_id, params, 0, 0)
    })
    .then(move |spring, result| {
        crate::engine::check(spring, result, "give_order_to_unit", unit_id)
    });
}
