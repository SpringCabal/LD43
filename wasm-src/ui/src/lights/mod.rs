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
    /// Keeps the staff light's flicker at its starting phase instead of
    /// following real time, so captures repeat.
    still: bool,
}

impl Lights {
    /// Lights whose flicker follows real time, as in the game, or stays
    /// `still`.
    pub fn new(still: bool) -> Self {
        Self {
            still,
            ..Self::default()
        }
    }

    /// Where the staff light's flicker is, in seconds.
    fn flicker_seconds(&self) -> f32 {
        if self.still {
            0.0
        } else {
            staff::clock_seconds()
        }
    }

    pub fn update(&mut self, global: &Global) {
        if self.houses.is_none() {
            self.houses = houses::place();
        }
        let status = *global.status.access();
        let seconds = self.flicker_seconds();
        self.staff = staff::light(&status, &mut self.staff_boost, seconds);
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
