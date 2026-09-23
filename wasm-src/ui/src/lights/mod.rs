//! The point lights the deferred lighting pass draws, ported from the Lua
//! build's house, unit, mouse and projectile light widgets.

mod houses;
mod mouse;
mod projectiles;
mod staff;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::state::Global;

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
}

/// Every light in the scene.
#[derive(Default)]
pub struct Lights {
    /// Window lights, placed once the houses exist.
    houses: Option<Vec<PointLight>>,
    staff: Option<PointLight>,
    /// Multiplies the staff light while the blood mage casts.
    staff_boost: staff::Boost,
    mouse: Option<PointLight>,
    projectiles: Vec<PointLight>,
    /// The light each weapon def's projectiles carry.
    projectile_defs: BTreeMap<i32, Option<projectiles::DefLight>>,
}

impl Lights {
    pub fn update(&mut self, global: &Global) {
        if self.houses.is_none() {
            self.houses = houses::place();
        }
        let status = *global.status.access();
        // The Lua widget flickers the staff light with os.clock(), which would
        // make every screenshot different, so the flicker stays at its
        // starting phase.
        self.staff = staff::light(&status, &mut self.staff_boost, 0.0);
        if let Some(ground) = global.cursor.get().ground {
            self.mouse = Some(mouse::light(ground));
        }
        projectiles::update(&mut self.projectiles, &mut self.projectile_defs);
    }

    pub fn iter(&self) -> impl Iterator<Item = &PointLight> {
        self.houses
            .iter()
            .flatten()
            .chain(&self.staff)
            .chain(&self.mouse)
            .chain(&self.projectiles)
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// What is lit, for the one-time log line.
    pub fn summary(&self) -> (bool, bool, usize) {
        (
            self.staff.is_some(),
            self.mouse.is_some(),
            self.projectiles.len(),
        )
    }
}
