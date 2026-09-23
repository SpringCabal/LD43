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

/// The living villagers, each tied to the area it guards.
#[derive(Default)]
pub struct Villagers {
    list: Vec<Villager>,
    cursor: usize,
}

impl Villagers {
    pub fn new(list: Vec<Villager>) -> Self {
        Self { list, cursor: 0 }
    }

    /// Forgets a dead villager; returns whether it was one.
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

    /// The next slice of villagers to check, round-robin, so every villager is
    /// checked once every `divisor` calls.
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

/// Sends an idle villager that has wandered out of its area back into it.
pub fn check_idle(villager: Villager) {
    let (unit_id, area) = (villager.unit_id, villager.area);
    if units::has_orders(unit_id) {
        return;
    }
    let Some(position) = units::position(unit_id) else {
        return;
    };
    if (position.x - area.x).abs() < area.width && (position.z - area.z).abs() < area.height {
        return;
    }
    let x = area.x + random::centered(area.width);
    let z = area.z + random::centered(area.height);
    let y = units::ground_height(x, z);
    let _ = spring::give_order_to_unit(unit_id, cmd::FIGHT, &[x, y, z], 0, 0);
}
