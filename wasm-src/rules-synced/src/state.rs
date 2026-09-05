use core::cell::RefCell;

use alloc::vec::Vec;

use crate::combat::{OrkAi, Story, Victory};
use crate::diagnostics;
use crate::player::Player;
use crate::town::VillagerArea;

pub struct World {
    pub initialized: bool,
    pub frame_consumed: bool,
    pub frame: i32,
    pub villagers: Vec<Villager>,
    pub villager_cursor: usize,
    pub villagers_killed: i32,
    pub allies_drained: i32,
    pub orks: OrkAi,
    pub story: Story,
    pub victory: Victory,
    pub player: Player,
}

#[derive(Clone, Copy)]
pub struct Villager {
    pub unit_id: i32,
    pub area: &'static VillagerArea,
}

thread_local! {
    static WORLD: RefCell<World> = RefCell::new(World::new());
}

pub fn with_world<R>(function: impl FnOnce(&mut World) -> R) -> Option<R> {
    WORLD.with(|world| match world.try_borrow_mut() {
        Ok(mut world) => Some(function(&mut world)),
        Err(_) => {
            diagnostics::warning("world state re-entered; skipping update");
            None
        }
    })
}

impl World {
    fn new() -> Self {
        Self {
            initialized: false,
            frame_consumed: false,
            frame: 0,
            villagers: Vec::new(),
            villager_cursor: 0,
            villagers_killed: 0,
            allies_drained: 0,
            orks: OrkAi::new(),
            story: Story::new(),
            victory: Victory::new(),
            player: Player::new(),
        }
    }

    pub fn game_end_state(&self) -> &'static str {
        if self.victory.declared() {
            "victory"
        } else if self.story.victory_possible() {
            "victoryPossible"
        } else {
            "start"
        }
    }

    pub fn remove_villager(&mut self, unit_id: i32) -> bool {
        let Some(index) = self
            .villagers
            .iter()
            .position(|villager| villager.unit_id == unit_id)
        else {
            return false;
        };
        self.villagers.swap_remove(index);
        if self.villager_cursor >= self.villagers.len() {
            self.villager_cursor = 0;
        }
        self.villagers_killed += 1;
        true
    }

    pub fn next_villagers(&mut self, count: usize) -> Vec<Villager> {
        let mut batch = Vec::with_capacity(count);
        for _ in 0..count {
            if self.villagers.is_empty() {
                break;
            }
            if self.villager_cursor >= self.villagers.len() {
                self.villager_cursor = 0;
            }
            batch.push(self.villagers[self.villager_cursor]);
            self.villager_cursor += 1;
        }
        batch
    }
}

#[derive(Default)]
pub struct Global;
