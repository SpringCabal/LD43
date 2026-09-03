use crate::units;
use crate::units::{ENEMY_TEAM, OrkKind};

pub struct Victory {
    declared: bool,
}

impl Victory {
    pub fn new() -> Self {
        Self { declared: false }
    }

    pub fn update(&mut self, possible: bool) -> bool {
        if self.declared || !possible || boss_alive() {
            return false;
        }
        self.declared = true;
        true
    }

    pub fn declared(&self) -> bool {
        self.declared
    }
}

fn boss_alive() -> bool {
    let Some(def_id) = units::def_id(OrkKind::Boss.def_name()) else {
        return true;
    };
    spring::get_team_unit_def_count(ENEMY_TEAM, def_id)
        .map(|count| count > 0)
        .unwrap_or(true)
}
