use crate::defs::{UnitKind, defs};
use crate::engine::set_unit_number;
use crate::units;

use super::spells::Spell;

#[derive(Clone, Copy)]
pub enum PlayerCommand {
    Move((f32, f32)),
    Attack { target: (f32, f32), move_to: bool },
    Stop,
    Cast { spell: Spell, x: f32, z: f32 },
}

pub struct Player {
    unit_id: Option<i32>,
    pending: Option<Pending>,
    move_goal: Option<MoveGoal>,
    attacking: bool,
    moving_to_attack: bool,
    casting_until: i32,
    cast_start_frame: i32,
    cast_animation_time: i32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            unit_id: None,
            pending: None,
            move_goal: None,
            attacking: false,
            moving_to_attack: false,
            casting_until: 0,
            cast_start_frame: 0,
            cast_animation_time: 0,
        }
    }

    pub fn unit_id(&self) -> Option<i32> {
        self.unit_id
    }

    pub fn adopt(&mut self, unit_id: i32, unit_def_id: i32) {
        if defs().is(unit_def_id, UnitKind::BloodMage) {
            self.unit_id = Some(unit_id);
        }
    }

    pub fn release(&mut self, unit_id: i32) {
        if self.unit_id == Some(unit_id) {
            self.unit_id = None;
        }
    }

    pub fn accept(&mut self, command: PlayerCommand, frame: i32) {
        if self.unit_id.is_none() || frame < self.casting_until {
            return;
        }
        if let Some(pending) = self.pending
            && pending.move_to
            && !pending.used
        {
            return;
        }
        self.pending = Some(Pending {
            command,
            frame,
            move_to: matches!(command, PlayerCommand::Attack { move_to: true, .. }),
            used: false,
        });
    }

    pub fn cast_start_frame(&self) -> i32 {
        self.cast_start_frame
    }

    pub fn cast_animation_time(&self) -> i32 {
        self.cast_animation_time
    }

    pub fn begin_cast(&mut self, frame: i32, frames: i32) {
        self.casting_until = frame + frames;
        self.cast_start_frame = frame;
        self.cast_animation_time = frames + 8;
        if let Some(unit_id) = self.unit_id {
            let _ = spring::clear_unit_goal(unit_id, true);
        }
        self.move_goal = None;
    }

    pub fn casting(&self, frame: i32) -> bool {
        frame < self.casting_until
    }

    pub fn take_cast(&mut self, frame: i32) -> Option<(Spell, f32, f32)> {
        let pending = self.pending?;
        if pending.frame + INPUT_WINDOW <= frame {
            return None;
        }
        let PlayerCommand::Cast { spell, x, z } = pending.command else {
            return None;
        };
        self.pending = None;
        Some((spell, x, z))
    }

    pub fn update(&mut self, frame: i32) {
        let Some(unit_id) = self.unit_id else {
            return;
        };
        if self.casting(frame) {
            return;
        }
        match self.pending {
            Some(pending) if pending.frame + INPUT_WINDOW > frame => {
                self.apply(unit_id, pending.command);
                if let Some(pending) = self.pending.as_mut() {
                    pending.used = true;
                }
            }
            _ => self.clear_attack(unit_id),
        }
        self.update_move_goal(unit_id);
    }
}

const INPUT_WINDOW: i32 = 2;
const GOAL_RADIUS: f32 = 60.0;
const ATTACK_RADIUS: f32 = 50.0;
const ATTACK_REACH: f32 = 50.0;
const AIM_HEIGHT: f32 = 25.0;
const MAP_MARGIN: f32 = 32.0;

#[derive(Clone, Copy)]
struct Pending {
    command: PlayerCommand,
    frame: i32,
    move_to: bool,
    used: bool,
}

#[derive(Clone, Copy)]
struct MoveGoal {
    x: f32,
    z: f32,
    radius: f32,
}

impl Player {
    fn apply(&mut self, unit_id: i32, command: PlayerCommand) {
        match command {
            PlayerCommand::Move((x, z)) => self.move_to(unit_id, x, z),
            PlayerCommand::Attack { target, move_to } => {
                self.attack(unit_id, target.0, target.1, move_to)
            }
            PlayerCommand::Stop => self.clear_move(unit_id),
            PlayerCommand::Cast { .. } => {}
        }
    }

    fn move_to(&mut self, unit_id: i32, x: f32, z: f32) {
        self.clear_attack(unit_id);
        self.set_move_goal(unit_id, x, z, GOAL_RADIUS);
    }

    fn attack(&mut self, unit_id: i32, x: f32, z: f32, move_to: bool) {
        let Some(position) = units::position(unit_id) else {
            return;
        };
        let dx = x - position.x;
        let dz = z - position.z;
        let distance = (dx * dx + dz * dz).sqrt().max(0.1);
        if move_to && distance > ATTACK_RADIUS {
            self.set_move_goal(unit_id, x, z, ATTACK_RADIUS);
            self.moving_to_attack = true;
            return;
        }
        if self.moving_to_attack {
            if self.move_goal.is_some() && distance > ATTACK_RADIUS {
                return;
            }
            self.moving_to_attack = false;
        }
        self.clear_move(unit_id);
        let tx = position.x + ATTACK_REACH * dx / distance;
        let tz = position.z + ATTACK_REACH * dz / distance;
        let ty = units::ground_height(tx, tz) + AIM_HEIGHT;
        let _ = spring::set_unit_target(
            unit_id,
            spring::types::UnitTargetRef {
                target_id: -1,
                pos: spring::types::Float3 {
                    x: tx,
                    y: ty,
                    z: tz,
                },
                is_ground_target: true,
            },
            spring::types::SetUnitTargetOptions {
                manual_fire: false,
                user_target: false,
            },
            0,
        );
        publish_fire_target(unit_id, Some((tx, tz)));
        self.attacking = true;
    }

    fn set_move_goal(&mut self, unit_id: i32, x: f32, z: f32, radius: f32) {
        let (x, z) = clamp_to_map(x, z);
        let y = units::ground_height(x, z);
        let _ =
            spring::set_unit_move_goal(unit_id, spring::types::Float3 { x, y, z }, 10.0, 0.0, true);
        self.move_goal = Some(MoveGoal { x, z, radius });
    }

    fn clear_move(&mut self, unit_id: i32) {
        if self.move_goal.is_some() {
            let _ = spring::clear_unit_goal(unit_id, true);
            self.move_goal = None;
        }
    }

    fn clear_attack(&mut self, unit_id: i32) {
        if self.attacking {
            let _ = spring::set_unit_target(
                unit_id,
                spring::types::UnitTargetRef {
                    target_id: -1,
                    pos: spring::types::Float3::default(),
                    is_ground_target: false,
                },
                spring::types::SetUnitTargetOptions {
                    manual_fire: false,
                    user_target: false,
                },
                0,
            );
            publish_fire_target(unit_id, None);
            self.attacking = false;
        }
    }

    fn update_move_goal(&mut self, unit_id: i32) {
        let Some(goal) = self.move_goal else {
            return;
        };
        let Some(position) = units::position(unit_id) else {
            return;
        };
        let dx = position.x - goal.x;
        let dz = position.z - goal.z;
        if dx * dx + dz * dz < goal.radius * goal.radius {
            self.clear_move(unit_id);
        }
    }
}

/// The blood mage's unit script decides whether to block its shot and which way
/// to face from these params, as the Lua build did from its GG.fireTx/fireTz
/// globals.
fn publish_fire_target(unit_id: i32, target: Option<(f32, f32)>) {
    let (firing, tx, tz) = match target {
        Some((tx, tz)) => (1.0, tx, tz),
        None => (0.0, 0.0, 0.0),
    };
    set_unit_number(unit_id, "firing", firing);
    set_unit_number(unit_id, "fireTx", tx);
    set_unit_number(unit_id, "fireTz", tz);
}

fn clamp_to_map(x: f32, z: f32) -> (f32, f32) {
    let Ok(map) = spring::game::get_game_map_info(0) else {
        return (x, z);
    };
    let width = map.map_size_x as f32;
    let height = map.map_size_z as f32;
    (
        x.clamp(MAP_MARGIN, (width - MAP_MARGIN).max(MAP_MARGIN)),
        z.clamp(MAP_MARGIN, (height - MAP_MARGIN).max(MAP_MARGIN)),
    )
}
