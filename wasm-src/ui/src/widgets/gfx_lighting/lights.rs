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

#[derive(Default)]
pub struct Lights {
    houses: houses::HouseLights,
    staff: Option<PointLight>,
    staff_boost: staff::Boost,
    mouse: Option<PointLight>,
    projectiles: Vec<PointLight>,
    projectile_defs: BTreeMap<i32, Option<projectiles::DefLight>>,
    still: bool,
}

impl Lights {
    pub fn new(still: bool) -> Self {
        Self {
            houses: houses::HouseLights::default(),
            still,
            ..Self::default()
        }
    }

    pub fn init(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        self.houses.init(spring);
    }

    pub fn unit_created(
        &mut self,
        spring: &spring::Spring<'_, spring::Ui>,
        unit_id: i32,
        def_id: i32,
    ) {
        self.houses.created(spring, unit_id, def_id);
    }

    pub fn unit_destroyed(&mut self, unit_id: i32) {
        self.houses.destroyed(unit_id);
    }

    fn flicker_seconds(&self, spring: &spring::Spring<'_, spring::Ui>) -> f32 {
        if self.still {
            0.0
        } else {
            staff::clock_seconds(spring)
        }
    }

    pub fn update(&mut self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        let status = *global.status.access();
        let seconds = self.flicker_seconds(spring);
        self.staff = staff::light(spring, &status, &mut self.staff_boost, seconds);
        if let Some(ground) = global.cursor.get().ground {
            self.mouse = Some(mouse::light(ground));
        }
        projectiles::update(spring, &mut self.projectiles, &mut self.projectile_defs);
    }

    pub fn iter(&self) -> impl Iterator<Item = &PointLight> {
        self.houses
            .iter()
            .chain(&self.staff)
            .chain(&self.mouse)
            .chain(&self.projectiles)
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn summary(&self) -> (bool, bool, usize) {
        (
            self.staff.is_some(),
            self.mouse.is_some(),
            self.projectiles.len(),
        )
    }
}
