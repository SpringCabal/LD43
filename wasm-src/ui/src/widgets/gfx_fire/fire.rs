mod emit;
mod particles;
mod render;

use alloc::vec::Vec;

use crate::game;

use particles::Field;

#[derive(Default)]
pub struct Fire {
    field: Field,
    emitted_frame: Option<i32>,
    houses: Vec<i32>,
    burning: Vec<i32>,
    next_check: usize,
}

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

    pub fn update(&mut self, spring: &spring::Spring<'_, spring::Ui>, delta_seconds: f32) {
        let frame = game::status_frame(spring);
        if self.emitted_frame.replace(frame) != Some(frame) {
            self.find_new_fires(spring);
            emit::emit(spring, &mut self.field, &self.burning);
        }
        self.field.advance(delta_seconds);
    }

    pub fn draw(&self, spring: &spring::Spring<'_, spring::Ui>) {
        render::draw(spring, &self.field);
    }

    fn find_new_fires(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        let count = self.houses.len().min(HOUSES_CHECKED_PER_PERIOD);
        for _ in 0..count {
            if self.next_check >= self.houses.len() {
                self.next_check = 0;
            }
            let unit_id = self.houses[self.next_check];
            if emit::on_fire(spring, unit_id) {
                self.houses.swap_remove(self.next_check);
                self.burning.push(unit_id);
            } else {
                self.next_check += 1;
            }
        }
    }
}
