use crate::state::Villager;
use crate::units;
use crate::{cmd, random};

use super::spawn::VillagerArea;

pub fn update(villager: Villager) {
    check_idle(villager.unit_id, villager.area);
}

fn check_idle(unit_id: i32, area: &VillagerArea) {
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
