//! Flames on burning houses, a cut-down port of the LUPS fire the Lua build
//! draws with gfx_lups_units_on_fire.lua.

mod emit;
mod particles;
mod render;

use alloc::vec::Vec;

use crate::game;

use particles::Field;

/// The flames and the houses that can burn.
#[derive(Default)]
pub struct Fire {
    field: Field,
    /// The status frame flames were last emitted for.
    emitted_frame: Option<i32>,
    /// Living houses that haven't caught fire; the only units that can.
    houses: Vec<i32>,
    /// Living houses on fire. A house burns until it dies.
    burning: Vec<i32>,
    /// Where in `houses` the next check for new fires starts.
    next_check: usize,
}

/// How many unburnt houses are checked for fire per status period (6
/// frames): the town's 107 houses take about 1.4 s, instead of all of them
/// being polled every period.
const HOUSES_CHECKED_PER_PERIOD: usize = 16;

impl Fire {
    pub fn add_house(&mut self, unit_id: i32) {
        if !self.houses.contains(&unit_id) && !self.burning.contains(&unit_id) {
            self.houses.push(unit_id);
        }
    }

    pub fn remove_house(&mut self, unit_id: i32) {
        self.houses.retain(|house| *house != unit_id);
        self.burning.retain(|house| *house != unit_id);
    }

    /// Emits flames once per status period and moves them along.
    pub fn update(&mut self, delta_seconds: f32) {
        let frame = game::status_frame();
        if self.emitted_frame.replace(frame) != Some(frame) {
            self.find_new_fires();
            emit::emit(&mut self.field, &self.burning);
        }
        self.field.advance(delta_seconds);
    }

    pub fn draw(&self) {
        render::draw(&self.field);
    }

    /// Checks the next few unburnt houses for the rules' `on_fire` flag.
    fn find_new_fires(&mut self) {
        let count = self.houses.len().min(HOUSES_CHECKED_PER_PERIOD);
        for _ in 0..count {
            if self.next_check >= self.houses.len() {
                self.next_check = 0;
            }
            let unit_id = self.houses[self.next_check];
            if emit::on_fire(unit_id) {
                self.houses.swap_remove(self.next_check);
                self.burning.push(unit_id);
            } else {
                self.next_check += 1;
            }
        }
    }
}
