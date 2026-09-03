use alloc::vec::Vec;

use crate::random;
use crate::units::{self, ENEMY_TEAM, OrkKind};

use super::attributes::{self, Effect, Modifier};

pub struct OrkAi {
    units: Vec<i32>,
    cursor: usize,
    killed: i32,
    fleeing: bool,
}

impl OrkAi {
    pub fn new() -> Self {
        Self {
            units: Vec::new(),
            cursor: 0,
            killed: 0,
            fleeing: false,
        }
    }

    pub fn created(&mut self, unit_id: i32, unit_def_id: i32, team: i32) -> bool {
        if team != ENEMY_TEAM || !is_ork(unit_def_id) {
            return false;
        }
        self.units.push(unit_id);
        true
    }

    pub fn destroyed(&mut self, unit_id: i32) -> Option<i32> {
        let index = self.units.iter().position(|id| *id == unit_id)?;
        self.units.swap_remove(index);
        if self.cursor >= self.units.len() {
            self.cursor = 0;
        }
        self.killed += 1;
        Some(self.killed)
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

    pub fn killed(&self) -> i32 {
        self.killed
    }

    pub fn all(&self) -> Vec<i32> {
        self.units.clone()
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
    if has_orders(unit_id) {
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
    let _ = spring::give_order_to_unit(unit_id, CMD_FIRE_STATE, &[0.0], 0, 0);
    attributes::add_effect(
        unit_id,
        Effect::Flee,
        Modifier {
            move_factor: 1.0,
            reload_factor: 0.0,
        },
    );
    let _ = spring::give_order_to_unit(unit_id, CMD_MOVE, &[cx, cy, cz], 0, 0);
}

const AIM_X: f32 = 5650.0;
const AIM_Z: f32 = 5720.0;
const AIM_VARIANCE: f32 = 600.0;
const AIM_REACHED_VARIANCE: f32 = 2000.0;
const FLEE_DISTANCE: f32 = 4000.0;
const UPDATE_DIVISOR: usize = 80;
const CMD_MOVE: i32 = 10;
const CMD_FIGHT: i32 = 16;
const CMD_FIRE_STATE: i32 = 45;

fn is_ork(unit_def_id: i32) -> bool {
    [OrkKind::Small, OrkKind::Big, OrkKind::Boss]
        .into_iter()
        .filter_map(|kind| units::def_id(kind.def_name()))
        .any(|id| id == unit_def_id)
}

fn fight(unit_id: i32, x: f32, z: f32) {
    let y = units::ground_height(x, z);
    let _ = spring::give_order_to_unit(unit_id, CMD_FIGHT, &[x, y, z], 0, 0);
}

fn has_orders(unit_id: i32) -> bool {
    // Ask for one command: a limit of zero returns an empty queue for every
    // unit, which made every idle check re-issue the same order.
    spring::units_commands::get_command_queue(unit_id, 1)
        .map(|commands| !commands.is_empty())
        .unwrap_or(true)
}
