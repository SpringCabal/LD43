use crate::random;
use crate::state::Villager;
use crate::units;

use super::spawn::VillagerArea;

const CMD_FIGHT: i32 = 16;

pub fn update(villager: Villager) {
    check_idle(villager.unit_id, villager.area);
}

fn check_idle(unit_id: i32, area: &VillagerArea) {
    if has_orders(unit_id) {
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
    let _ = spring::give_order_to_unit(unit_id, CMD_FIGHT, &[x, y, z], 0, 0);
}

fn has_orders(unit_id: i32) -> bool {
    // Ask for one command: a limit of zero returns an empty queue for every
    // unit, which made every idle check re-issue the same order.
    spring::units_commands::get_command_queue(unit_id, 1)
        .map(|commands| !commands.is_empty())
        .unwrap_or(true)
}
