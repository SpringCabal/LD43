use spring::commands;
use spring::types::Float3;

use crate::defs::{UnitKind, defs};
use crate::units;

use super::spells::Spell;

#[derive(Clone, Copy)]
pub enum PlayerCommand {
    Move((f32, f32)),
    Attack { target: (f32, f32), move_to: bool },
    Stop,
    Cast { spell: Spell, x: f32, z: f32 },
}

#[derive(Default)]
pub struct Player {
    unit_id: Option<i32>,
    /// The move or attack the UI holds a mouse button for, until it says stop.
    held: Option<Held>,
    /// A spell cast between frames, cast on the next one.
    pending_cast: Option<PendingCast>,
    move_goal: Option<MoveGoal>,
    attacking: bool,
    moving_to_attack: bool,
    casting_until: i32,
    cast_start_frame: i32,
    /// How long the cast animation (and the staff glow) lasts, in frames.
    cast_animation_frames: i32,
}

impl Player {
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

    /// Takes a command from the UI. The UI sends a held move or attack when it
    /// starts and when its target moves, and `Stop` when the button is let go;
    /// the player keeps applying the held command every frame in between, as
    /// the Lua build did with the UI's per-frame messages.
    pub fn accept(&mut self, command: PlayerCommand, frame: i32) {
        let Some(unit_id) = self.unit_id else {
            return;
        };
        match command {
            PlayerCommand::Cast { spell, x, z } => {
                if !self.casting(frame) {
                    self.pending_cast = Some(PendingCast { spell, x, z, frame });
                }
            }
            PlayerCommand::Stop => {
                self.held = None;
                self.moving_to_attack = false;
                if !self.casting(frame) {
                    self.clear_move(unit_id);
                    self.clear_attack(unit_id);
                }
            }
            PlayerCommand::Move(_) | PlayerCommand::Attack { .. } => {
                // A move-to-attack runs at least once before anything replaces it.
                if self.held.is_some_and(|held| held.move_to && !held.used) {
                    return;
                }
                self.held = Some(Held {
                    command,
                    move_to: matches!(command, PlayerCommand::Attack { move_to: true, .. }),
                    used: false,
                });
            }
        }
    }

    /// Forgets the held move or attack without stopping what the unit does.
    pub fn drop_held(&mut self) {
        self.held = None;
    }

    pub fn begin_cast(&mut self, frame: i32, frames: i32) {
        self.casting_until = frame + frames;
        self.cast_start_frame = frame;
        self.cast_animation_frames = frames + CAST_ANIMATION_EXTRA_FRAMES;
        if let Some(unit_id) = self.unit_id {
            clear_goal(unit_id);
        }
        self.move_goal = None;
    }

    pub fn cast_start_frame(&self) -> i32 {
        self.cast_start_frame
    }

    pub fn cast_animation_frames(&self) -> i32 {
        self.cast_animation_frames
    }

    pub fn casting(&self, frame: i32) -> bool {
        frame < self.casting_until
    }

    pub fn take_cast(&mut self, frame: i32) -> Option<(Spell, f32, f32)> {
        let cast = self.pending_cast.take()?;
        (cast.frame + INPUT_WINDOW > frame).then_some((cast.spell, cast.x, cast.z))
    }

    pub fn update(&mut self, frame: i32) {
        let Some(unit_id) = self.unit_id else {
            return;
        };
        if self.casting(frame) {
            return;
        }
        match self.held.as_mut() {
            Some(held) => {
                held.used = true;
                let command = held.command;
                self.apply(unit_id, command);
            }
            None => self.clear_attack(unit_id),
        }
        self.update_move_goal(unit_id);
    }
}

/// A cast is dropped if no frame took it within this many frames.
const INPUT_WINDOW: i32 = 2;
/// The cast animation outlasts the cast by this many frames.
const CAST_ANIMATION_EXTRA_FRAMES: i32 = 8;
const GOAL_RADIUS: f32 = 60.0;
const ATTACK_RADIUS: f32 = 50.0;
const ATTACK_REACH: f32 = 50.0;
const AIM_HEIGHT: f32 = 25.0;
const MAP_MARGIN: f32 = 32.0;

#[derive(Clone, Copy)]
struct Held {
    command: PlayerCommand,
    move_to: bool,
    /// Whether it has been applied at least once.
    used: bool,
}

#[derive(Clone, Copy)]
struct PendingCast {
    spell: Spell,
    x: f32,
    z: f32,
    frame: i32,
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
        aim(
            unit_id,
            Some(Float3 {
                x: tx,
                y: ty,
                z: tz,
            }),
        );
        self.attacking = true;
    }

    fn set_move_goal(&mut self, unit_id: i32, x: f32, z: f32, radius: f32) {
        let (x, z) = clamp_to_map(x, z);
        let y = units::ground_height(x, z);
        set_goal(unit_id, Float3 { x, y, z });
        self.move_goal = Some(MoveGoal { x, z, radius });
    }

    fn clear_move(&mut self, unit_id: i32) {
        if self.move_goal.is_some() {
            clear_goal(unit_id);
            self.move_goal = None;
        }
    }

    fn clear_attack(&mut self, unit_id: i32) {
        if self.attacking {
            aim(unit_id, None);
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

// The player's engine calls run once the current callin has returned, in the
// order the player decided on them: after the blood magic of a cast, and with
// the player free for the callins they cause (the scripts' StartMoving and
// StopMoving).

/// Aims at a ground point, or stops aiming.
fn aim(unit_id: i32, target: Option<Float3>) {
    commands::queue(move || {
        spring::set_unit_target(
            unit_id,
            spring::types::UnitTargetRef {
                target_id: -1,
                pos: target.unwrap_or_default(),
                is_ground_target: target.is_some(),
            },
            spring::types::SetUnitTargetOptions {
                manual_fire: false,
                user_target: false,
            },
            0,
        )
    })
    .then(move |result| {
        crate::engine::check(result, "set_unit_target", unit_id);
        publish_fire_target(unit_id, target.map(|target| (target.x, target.z)));
    });
}

fn set_goal(unit_id: i32, goal: Float3) {
    commands::queue(move || spring::set_unit_move_goal(unit_id, goal, 10.0, 0.0, true))
        .then(move |result| crate::engine::check(result, "set_unit_move_goal", unit_id));
}

fn clear_goal(unit_id: i32) {
    commands::queue(move || spring::clear_unit_goal(unit_id, true))
        .then(move |result| crate::engine::check(result, "clear_unit_goal", unit_id));
}

/// The blood mage's unit script holds its fire without a target, and faces
/// it when it fires, as the Lua build's script did from GG.fireTx/fireTz.
fn publish_fire_target(unit_id: i32, target: Option<(f32, f32)>) {
    crate::unit_scripts::call(unit_id, crate::unit_scripts::ScriptCall::FireTarget(target));
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
