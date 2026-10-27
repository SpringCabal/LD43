mod emit;
mod particles;
#[cfg(feature = "fire-probe")]
pub mod probe;
mod render;
use alloc::vec::Vec;
use particles::{Batch, Field};

#[derive(Default)]
pub struct Fire {
    field: Field,
    batch: Batch,
    last_frame: i32,
    last_poll: i32,
    houses: Vec<i32>,
    burning: Vec<i32>,
    random: emit::Random,
}
fn insert(units: &mut Vec<i32>, id: i32) {
    if let Err(at) = units.binary_search(&id) {
        units.insert(at, id);
    }
}
fn remove(units: &mut Vec<i32>, id: i32) {
    if let Ok(at) = units.binary_search(&id) {
        units.remove(at);
    }
}
impl Fire {
    pub fn add_house(&mut self, id: i32) {
        if self.burning.binary_search(&id).is_err() {
            insert(&mut self.houses, id);
        }
    }
    pub fn remove_house(&mut self, id: i32) {
        remove(&mut self.houses, id);
        remove(&mut self.burning, id);
    }
    pub fn update(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        let frame = spring.game().current_frame().unwrap_or(0);
        if frame == self.last_frame {
            return;
        }
        self.field.advance((frame - self.last_frame).max(0) as f32);
        self.last_frame = frame;
        let poll = frame - frame % 6;
        if poll <= self.last_poll {
            return;
        }
        self.last_poll = poll;
        self.houses.retain(|id| {
            if emit::on_fire(spring, *id) {
                insert(&mut self.burning, *id);
                false
            } else {
                true
            }
        });
        emit::emit(spring, &mut self.field, &self.burning, &mut self.random);
    }
    pub fn draw(&self, spring: &spring::Spring<'_, spring::Ui>) {
        render::draw(spring, &self.field, &self.batch);
    }
    pub fn shutdown(&self, spring: &spring::Spring<'_, spring::Ui>) {
        self.batch.shutdown(spring);
    }
}
