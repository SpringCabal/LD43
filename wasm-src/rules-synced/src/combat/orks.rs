use alloc::vec::Vec;

use spring::cmd;

use crate::defs::defs;
use crate::random;
use crate::units::{self, ENEMY_TEAM};

use super::attributes::{self, Effect, Modifier};

#[derive(Default)]
pub struct OrkAi {
    units: Vec<i32>,
    cursor: usize,
    fleeing: bool,
}

impl OrkAi {
    pub fn created(&mut self, unit_id: i32, unit_def_id: i32, team: i32) -> bool {
        if team != ENEMY_TEAM || !is_ork(unit_def_id) {
            return false;
        }
        self.units.push(unit_id);
        true
    }

    /// Forgets a dead ork; returns whether it was one.
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

pub fn advance(unit_id: i32) {
    let dx = AIM_X + random::centered(AIM_VARIANCE);
    let dz = AIM_Z + random::centered(AIM_VARIANCE);
    fight(unit_id, dx, dz);
}

pub fn check_idle(unit_id: i32) {
    if units::has_orders(unit_id) {
        return;
    }
    let dx = AIM_X + random::centered(AIM_REACHED_VARIANCE);
    let dz = AIM_Z + random::centered(AIM_REACHED_VARIANCE);
    fight(unit_id, dx, dz);
}

pub fn flee(unit_id: i32) {
    let Some(position) = units::position(unit_id) else {
        return;
    };
    let dx = position.x - AIM_X;
    let dz = position.z - AIM_Z;
    let distance = (dx * dx + dz * dz).sqrt().max(0.1);
    let cx = position.x + FLEE_DISTANCE * dx / distance;
    let cz = position.z + FLEE_DISTANCE * dz / distance;
    let cy = units::ground_height(cx, cz);
    let _ = spring::give_order_to_unit(unit_id, cmd::FIRE_STATE, &[0.0], 0, 0);
    attributes::add_effect(
        unit_id,
        Effect::Flee,
        Modifier {
            move_factor: 1.0,
            reload_factor: 0.0,
        },
    );
    let _ = spring::give_order_to_unit(unit_id, cmd::MOVE, &[cx, cy, cz], 0, 0);
}

// Orks attack the town centre.
const AIM_X: f32 = units::TOWN_CENTRE_X;
const AIM_Z: f32 = units::TOWN_CENTRE_Z;
const AIM_VARIANCE: f32 = 600.0;
const AIM_REACHED_VARIANCE: f32 = 2000.0;
const FLEE_DISTANCE: f32 = 4000.0;
const UPDATE_DIVISOR: usize = 80;

fn is_ork(unit_def_id: i32) -> bool {
    defs().is_ork(unit_def_id)
}

fn fight(unit_id: i32, x: f32, z: f32) {
    let y = units::ground_height(x, z);
    let _ = spring::give_order_to_unit(unit_id, cmd::FIGHT, &[x, y, z], 0, 0);
}
