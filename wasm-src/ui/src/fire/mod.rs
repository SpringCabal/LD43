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
    /// Living houses, the only units that catch fire.
    houses: Vec<i32>,
}

impl Fire {
    pub fn add_house(&mut self, unit_id: i32) {
        if !self.houses.contains(&unit_id) {
            self.houses.push(unit_id);
        }
    }

    pub fn remove_house(&mut self, unit_id: i32) {
        self.houses.retain(|house| *house != unit_id);
    }

    /// Emits flames once per status period and moves them along.
    pub fn update(&mut self, delta_seconds: f32) {
        let frame = game::status_frame();
        if self.emitted_frame.replace(frame) != Some(frame) {
            emit::emit(&mut self.field, &self.houses);
        }
        self.field.advance(delta_seconds);
    }

    pub fn draw(&self) {
        render::draw(&self.field);
    }
}
