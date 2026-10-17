use alloc::vec::Vec;

use spring::cmd;

use crate::random;
use crate::units;

use super::spawn::VillagerArea;

#[derive(Clone, Copy)]
pub struct Villager {
    pub unit_id: i32,
    pub area: &'static VillagerArea,
}

#[derive(Default)]
pub struct Villagers {
    list: Vec<Villager>,
    cursor: usize,
}

impl Villagers {
    pub fn push(&mut self, villager: Villager) {
        self.list.push(villager);
    }

    pub fn count(&self) -> usize {
        self.list.len()
    }

    pub fn remove(&mut self, unit_id: i32) -> bool {
        let Some(index) = self
            .list
            .iter()
            .position(|villager| villager.unit_id == unit_id)
        else {
            return false;
        };
        self.list.swap_remove(index);
        if self.cursor >= self.list.len() {
            self.cursor = 0;
        }
        true
    }

    pub fn next_batch(&mut self, divisor: usize) -> Vec<Villager> {
        let count = self.list.len().div_ceil(divisor);
        let mut batch = Vec::with_capacity(count);
        for _ in 0..count {
            if self.list.is_empty() {
                break;
            }
            if self.cursor >= self.list.len() {
                self.cursor = 0;
            }
            batch.push(self.list[self.cursor]);
            self.cursor += 1;
        }
        batch
    }
}

pub fn check_idle(spring: &spring::Spring<'_, spring::RulesSynced>, villager: Villager) {
    let (unit_id, area) = (villager.unit_id, villager.area);
    if units::has_orders(spring, unit_id) {
        return;
    }
    let Some(position) = units::position(spring, unit_id) else {
        return;
    };
    if (position.x - area.x).abs() < area.width && (position.z - area.z).abs() < area.height {
        return;
    }
    let x = area.x + random::centered(spring, area.width);
    let z = area.z + random::centered(spring, area.height);
    let y = units::ground_height(spring, x, z);
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .give_order_to_unit(unit_id, cmd::FIGHT, &[x, y, z], 0, 0)
    })
    .then(move |spring, result| {
        crate::engine::check(spring, result, "give_order_to_unit", unit_id)
    });
}
