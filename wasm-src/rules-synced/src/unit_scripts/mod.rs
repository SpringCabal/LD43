//! TTLD's unit scripts (the Lua build's `scripts/*.lua`) as Core-CUS callbacks
//! and tasks. The unit definitions still select the models and weapons.

mod actions;
mod common;
mod kinds;
mod module;
mod tasks;

pub use module::{TtldCusModule, queue_call, queue_unit_created};

use alloc::rc::Rc;
use core::cell::RefCell;

use spring::UnitId;
use spring::cus::{
    Axis, InitCtx, Piece, TaskDefinition, UnitCtx, UnitScript, WeaponId, WreckLevel,
};

use crate::combat;
use crate::engine::{self, set_unit_number};

use self::actions::{coward_block, face_from_position, issue_patrol_orders, ork_toss};
use self::common::{
    SIG_ATTACK, SIG_BUFF, SIG_MOVE, SIG_STUN, collision_volume, face_target, move_piece,
    play_unit_sound, random_unit, set_blocking,
};
use self::kinds::ScriptKind;
use self::tasks::{
    attack_bob_task, attack_task, bob_task, boss_position_task, buff_task, cast_task, death_task,
    house_death_task, stun_task,
};

const COB_INBUILDSTANCE: i32 = 5;

#[derive(Clone, Copy)]
pub(super) struct Pieces {
    hand_right: Piece,
    hand_left: Piece,
    torso: Piece,
    house: Piece,
}

impl Pieces {
    fn from_init(ctx: &InitCtx<'_>) -> Self {
        Self {
            hand_right: ctx.piece("Hand_Right"),
            hand_left: ctx.piece("Hand_Left"),
            torso: ctx.piece("Torso"),
            house: ctx.piece("House"),
        }
    }
}

pub(super) struct TtldState {
    kind: Option<ScriptKind>,
    pieces: Pieces,
    dead: bool,
    death_pending: bool,
    first_bob_run: bool,
    bob_scale: f32,
    static_bob_scale: f32,
    time_scale: f32,
    size_scale: f32,
    is_ork: bool,
    death_sound: Option<&'static str>,
    death_sound_chance: f32,
    toss_radius: f32,
    toss_strength: f32,
    deunorkable: bool,
    cast_level: i32,
    cast_tx: f32,
    cast_tz: f32,
    random: u32,
}

impl TtldState {
    fn new(ctx: &mut InitCtx<'_>) -> Self {
        let unit = ctx.unit();
        Self {
            kind: None,
            pieces: Pieces::from_init(ctx),
            dead: false,
            death_pending: false,
            first_bob_run: true,
            bob_scale: 2.0,
            static_bob_scale: 2.0,
            time_scale: 1.0,
            size_scale: 1.0,
            is_ork: false,
            death_sound: None,
            death_sound_chance: 1.0,
            toss_radius: 0.0,
            toss_strength: 0.0,
            deunorkable: false,
            cast_level: 1,
            cast_tx: 0.0,
            cast_tz: 0.0,
            random: unit.0 as u32 ^ 0x5454_4c44,
        }
    }
}

pub struct TtldUnitScript {
    kind: Option<ScriptKind>,
    state: Rc<RefCell<TtldState>>,
}

impl UnitScript for TtldUnitScript {
    fn new(ctx: &mut InitCtx<'_>) -> Self {
        Self {
            kind: None,
            state: Rc::new(RefCell::new(TtldState::new(ctx))),
        }
    }

    fn create(&mut self, ctx: &UnitCtx) {
        let Some(kind) = self.kind else {
            return;
        };
        match kind {
            ScriptKind::BloodMage => {
                self.init_shared(ctx, 2.0, 1.0, false, Some("sounds/death_player.wav"), 1.0);
                set_unit_number(ctx.unit().0, "unorkable", 1.0);
            }
            ScriptKind::Coward => {
                self.init_shared(ctx, 3.0, 1.0, false, Some("sounds/panicdeathyell.wav"), 0.6);
            }
            ScriptKind::Crossbowman => {
                self.init_shared(ctx, 3.0, 1.0, false, Some("sounds/bravedeathyell.wav"), 0.6);
                issue_patrol_orders(ctx);
            }
            ScriptKind::Peasant => {
                self.init_shared(ctx, 3.0, 1.0, false, Some("sounds/panicdeathyell.wav"), 0.6);
                issue_patrol_orders(ctx);
            }
            ScriptKind::Swordsman => {
                self.init_shared(ctx, 3.0, 1.0, false, Some("sounds/bravedeathyell.wav"), 0.6);
                issue_patrol_orders(ctx);
            }
            ScriptKind::OrkSmall => {
                self.init_shared(
                    ctx,
                    3.0,
                    0.9,
                    true,
                    Some("sounds/smallorkdeathyell.wav"),
                    0.6,
                );
                set_unit_number(ctx.unit().0, "unorkable", 1.0);
                let _ = spring::set_unit_max_range(ctx.unit().0, 80.0);
            }
            ScriptKind::OrkBig => {
                self.init_shared(ctx, 2.0, 2.0, true, Some("sounds/bigorkdeathyell.wav"), 1.0);
                set_unit_number(ctx.unit().0, "unorkable", 1.0);
                self.state.borrow_mut().toss_radius = 120.0;
                self.state.borrow_mut().toss_strength = 5.0;
                let _ = spring::set_unit_max_range(ctx.unit().0, 120.0);
            }
            ScriptKind::OrkBoss => {
                self.init_shared(ctx, 2.0, 2.0, true, Some("sounds/bossdeathyell.wav"), 1.0);
                set_unit_number(ctx.unit().0, "unorkable", 1.0);
                {
                    let mut state = self.state.borrow_mut();
                    state.toss_radius = 200.0;
                    state.toss_strength = 10.0;
                    state.deunorkable = true;
                }
                let _ = spring::set_unit_max_range(ctx.unit().0, 200.0);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-ork-boss-position",
                    Rc::clone(&self.state),
                    boss_position_task,
                ));
            }
            ScriptKind::House => {}
        }
    }

    fn start_moving(&mut self, ctx: &UnitCtx, _reversing: bool) {
        if self.state.borrow().dead {
            return;
        }
        ctx.signal(SIG_MOVE);
        ctx.spawn(TaskDefinition::with_state(
            "ttld-bob",
            Rc::clone(&self.state),
            bob_task,
        ));
    }

    fn stop_moving(&mut self, ctx: &UnitCtx) {
        if self.state.borrow().dead {
            return;
        }
        ctx.signal(SIG_MOVE);
        let piece = self.state.borrow().pieces.torso;
        move_piece(ctx, piece, Axis::Y, 0.0, 10.0);
    }

    fn start_building(&mut self, ctx: &UnitCtx) {
        if matches!(
            self.kind,
            Some(
                ScriptKind::Coward
                    | ScriptKind::Crossbowman
                    | ScriptKind::Peasant
                    | ScriptKind::Swordsman
            )
        ) {
            ctx.set_unit_value(1, COB_INBUILDSTANCE);
        }
    }

    fn query_weapon(&mut self, _ctx: &UnitCtx, _weapon: WeaponId) -> Piece {
        match self.kind {
            Some(
                ScriptKind::BloodMage
                | ScriptKind::Crossbowman
                | ScriptKind::Peasant
                | ScriptKind::Swordsman,
            ) => self.state.borrow().pieces.hand_right,
            Some(
                ScriptKind::Coward
                | ScriptKind::OrkSmall
                | ScriptKind::OrkBig
                | ScriptKind::OrkBoss,
            ) => self.state.borrow().pieces.torso,
            _ => Piece::INVALID,
        }
    }

    fn aim_from_weapon(&mut self, ctx: &UnitCtx, weapon: WeaponId) -> Piece {
        if weapon.0 < 0 {
            return Piece::INVALID;
        }
        self.query_weapon(ctx, weapon)
    }

    fn aim_weapon(
        &mut self,
        _ctx: &UnitCtx,
        _weapon: WeaponId,
        _heading: f32,
        _pitch: f32,
    ) -> Option<bool> {
        match self.kind {
            Some(
                ScriptKind::BloodMage
                | ScriptKind::Coward
                | ScriptKind::Crossbowman
                | ScriptKind::Peasant
                | ScriptKind::Swordsman
                | ScriptKind::OrkSmall
                | ScriptKind::OrkBig
                | ScriptKind::OrkBoss,
            ) => Some(true),
            _ => None,
        }
    }

    fn block_shot(
        &mut self,
        ctx: &UnitCtx,
        weapon: WeaponId,
        target: UnitId,
        _user_target: bool,
    ) -> bool {
        let unit = ctx.unit();
        match self.kind {
            Some(ScriptKind::Coward) => coward_block(ctx, target),
            Some(ScriptKind::Swordsman) if weapon.0 == 1 => {
                let _ = spring::set_unit_weapon_state(
                    unit.0,
                    weapon.0,
                    "reloadFrame",
                    engine::frame() as f32 + 300.0,
                );
                true
            }
            Some(ScriptKind::BloodMage) => {
                let firing = spring::get_unit_rules_param_f32(unit.0, "firing").unwrap_or(0.0);
                if firing <= 0.0 {
                    return true;
                }
                if let (Some(tx), Some(tz)) = (
                    spring::get_unit_rules_param_f32(unit.0, "fireTx").ok(),
                    spring::get_unit_rules_param_f32(unit.0, "fireTz").ok(),
                ) {
                    face_from_position(unit, tx, tz);
                }
                false
            }
            Some(ScriptKind::OrkSmall | ScriptKind::OrkBig | ScriptKind::OrkBoss) => {
                face_target(unit, target.0);
                ork_toss(unit, target.0, &self.state);
                false
            }
            Some(ScriptKind::Crossbowman | ScriptKind::Peasant | ScriptKind::Swordsman) => {
                face_target(unit, target.0);
                false
            }
            _ => false,
        }
    }

    fn fire_weapon(&mut self, ctx: &UnitCtx, weapon: WeaponId) {
        match self.kind {
            Some(ScriptKind::BloodMage) => {
                if let (Some(tx), Some(tz)) = (
                    spring::get_unit_rules_param_f32(ctx.unit().0, "fireTx").ok(),
                    spring::get_unit_rules_param_f32(ctx.unit().0, "fireTz").ok(),
                ) {
                    face_from_position(ctx.unit(), tx, tz);
                }
                let _ = spring::clear_unit_goal(ctx.unit().0, true);
                ctx.signal(SIG_ATTACK);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-blood-attack",
                    Rc::clone(&self.state),
                    attack_task,
                ));
            }
            Some(ScriptKind::OrkSmall | ScriptKind::OrkBig | ScriptKind::OrkBoss)
                if weapon.0 == 0 =>
            {
                ctx.signal(SIG_ATTACK);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-ork-attack-bob",
                    Rc::clone(&self.state),
                    attack_bob_task,
                ));
            }
            Some(ScriptKind::Crossbowman | ScriptKind::Peasant | ScriptKind::Swordsman) => {
                let _ = spring::clear_unit_goal(ctx.unit().0, true);
                ctx.signal(SIG_ATTACK);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-hand-attack",
                    Rc::clone(&self.state),
                    attack_task,
                ));
            }
            _ => {}
        }
    }

    fn hit_by_weapon(
        &mut self,
        ctx: &UnitCtx,
        _hit_direction: spring::Float3,
        _weapon: spring::WeaponDefId,
        damage: f32,
    ) -> f32 {
        if self.kind == Some(ScriptKind::House) && damage > 400.0 {
            set_unit_number(ctx.unit().0, "on_fire", 1.0);
        }
        damage
    }

    fn killed(&mut self, ctx: &UnitCtx, _recent_damage: f32, _max_health: f32) -> WreckLevel {
        let Some(kind) = self.kind else {
            return WreckLevel::None;
        };
        if self.state.borrow().death_pending {
            return WreckLevel::NoWreck;
        }
        ctx.signal(SIG_MOVE);
        ctx.signal(SIG_BUFF);
        ctx.signal(SIG_STUN);
        ctx.signal(SIG_ATTACK);
        self.state.borrow_mut().dead = true;
        self.state.borrow_mut().death_pending = true;
        match kind {
            ScriptKind::House => {
                collision_volume(ctx.unit(), false);
                collision_volume(ctx.unit(), true);
                let _ = spring::set_unit_neutral(ctx.unit().0, true);
                let state = Rc::clone(&self.state);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-house-death",
                    state,
                    house_death_task,
                ));
            }
            _ => {
                let state = Rc::clone(&self.state);
                let sound = self.state.borrow().death_sound;
                let chance = self.state.borrow().death_sound_chance;
                if sound.is_some() && random_unit(&mut self.state.borrow_mut().random) < chance {
                    play_unit_sound(ctx.unit(), sound.unwrap_or_default(), 10.0);
                }
                collision_volume(ctx.unit(), false);
                collision_volume(ctx.unit(), true);
                let _ = spring::set_unit_neutral(ctx.unit().0, true);
                // Safe inside a CUS callin: the UnitGiven/UnitTaken this raises
                // have no handler in this module, and the attribute store is
                // never borrowed across an engine call.
                let _ =
                    spring::transfer_unit(ctx.unit().0, crate::units::NEUTRAL_TEAM, false, false);
                combat::add_effect(
                    ctx.unit().0,
                    combat::Effect::DeathParalysis,
                    combat::Modifier {
                        move_factor: 0.0,
                        reload_factor: 0.0,
                    },
                );
                set_blocking(ctx.unit(), false);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-human-death",
                    state,
                    death_task,
                ));
            }
        }
        WreckLevel::NoWreck
    }

    fn killed_pending(&self) -> bool {
        self.state.borrow().death_pending
    }
}

impl TtldUnitScript {
    fn set_kind(&mut self, kind: ScriptKind) {
        self.kind = Some(kind);
        self.state.borrow_mut().kind = Some(kind);
    }

    fn init_shared(
        &mut self,
        _ctx: &UnitCtx,
        bob_scale: f32,
        size_scale: f32,
        is_ork: bool,
        sound: Option<&'static str>,
        chance: f32,
    ) {
        let mut state = self.state.borrow_mut();
        state.bob_scale = bob_scale;
        state.static_bob_scale = bob_scale;
        state.size_scale = size_scale;
        state.is_ork = is_ork;
        state.death_sound = sound;
        state.death_sound_chance = chance;
    }

    fn named_call(&mut self, ctx: &UnitCtx, name: &str, arguments: &[f32]) -> bool {
        if self.state.borrow().dead {
            return matches!(name, "StopMoving");
        }
        match name {
            "StopMoving" => {
                self.stop_moving(ctx);
                true
            }
            "Buff" => {
                ctx.signal(SIG_BUFF);
                self.state.borrow_mut().cast_level =
                    arguments.first().copied().unwrap_or(0.0) as i32;
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-buff",
                    Rc::clone(&self.state),
                    buff_task,
                ));
                true
            }
            "Stun" => {
                ctx.signal(SIG_STUN);
                self.state.borrow_mut().cast_level =
                    arguments.first().copied().unwrap_or(0.0) as i32;
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-stun",
                    Rc::clone(&self.state),
                    stun_task,
                ));
                true
            }
            "CastAnimation" if self.kind == Some(ScriptKind::BloodMage) => {
                let mut state = self.state.borrow_mut();
                state.cast_level = (arguments.first().copied().unwrap_or(1.0) as i32).clamp(1, 3);
                state.cast_tx = arguments.get(1).copied().unwrap_or(0.0);
                state.cast_tz = arguments.get(2).copied().unwrap_or(0.0);
                drop(state);
                ctx.signal(SIG_ATTACK);
                ctx.spawn(TaskDefinition::with_state(
                    "ttld-cast",
                    Rc::clone(&self.state),
                    cast_task,
                ));
                true
            }
            _ => false,
        }
    }
}
