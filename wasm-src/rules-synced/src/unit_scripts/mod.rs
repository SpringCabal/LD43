//! TTLD's unit scripts (the Lua build's `scripts/*.lua`) as Core-CUS callbacks
//! and tasks. The unit definitions still select the models and weapons.

mod common;

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::UnitId;
use spring::cus::core_module::{CoreCusCallResult, CoreCusModule};
use spring::cus::wasm::WasmCus;
use spring::cus::{
    Axis, CusHandle, CusInstance, CusRegistry, Duration, InitCtx, Piece, ScriptCapabilities,
    TaskDefinition, UnitCtx, UnitScript, UnitScriptCall, WeaponId, WreckLevel,
};

use crate::engine::{self, set_unit_number};
use crate::{cmd, combat};

use self::common::{
    EnginePieces, SIG_ATTACK, SIG_BUFF, SIG_DEATH, SIG_MOVE, SIG_STUN, collision_volume,
    face_direction, face_target, move_piece, play_unit_sound, random_unit, set_blocking, spin,
    stop_spin, turn,
};

const COB_INBUILDSTANCE: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptKind {
    BloodMage,
    Coward,
    Crossbowman,
    House,
    OrkSmall,
    OrkBig,
    OrkBoss,
    Peasant,
    Swordsman,
}

fn script_kind_for_def(def: i32) -> Option<ScriptKind> {
    let matches = |name: &str| spring::get_unit_def_id_by_name(name).ok() == Some(def);
    if matches("bloodmage") {
        Some(ScriptKind::BloodMage)
    } else if matches("house") {
        Some(ScriptKind::House)
    } else if matches("orksmall") {
        Some(ScriptKind::OrkSmall)
    } else if matches("orkbig") {
        Some(ScriptKind::OrkBig)
    } else if matches("orkboss") {
        Some(ScriptKind::OrkBoss)
    } else if matches("coward") {
        Some(ScriptKind::Coward)
    } else if matches("crossbowman") || matches("builder") {
        // The Lua build names defender.lua for the builder, a file it never
        // shipped. The builder has the crossbowman's model and weapon, so it
        // gets the crossbowman's script.
        Some(ScriptKind::Crossbowman)
    } else if matches("swordsman") {
        Some(ScriptKind::Swordsman)
    } else if matches("peasant") {
        Some(ScriptKind::Peasant)
    } else {
        None
    }
}

fn capabilities(kind: ScriptKind) -> ScriptCapabilities {
    let mut bits = ScriptCapabilities::KILLED | ScriptCapabilities::CREATE;
    match kind {
        ScriptKind::House => bits |= ScriptCapabilities::HIT_BY_WEAPON,
        ScriptKind::Coward => {
            bits |= ScriptCapabilities::START_MOVING
                | ScriptCapabilities::STOP_MOVING
                | ScriptCapabilities::START_BUILDING
                | ScriptCapabilities::QUERY_WEAPON
                | ScriptCapabilities::AIM_WEAPON
                | ScriptCapabilities::AIM_FROM_WEAPON
                | ScriptCapabilities::BLOCK_SHOT;
        }
        ScriptKind::Crossbowman | ScriptKind::Peasant | ScriptKind::Swordsman => {
            bits |= ScriptCapabilities::START_MOVING
                | ScriptCapabilities::STOP_MOVING
                | ScriptCapabilities::START_BUILDING
                | ScriptCapabilities::QUERY_WEAPON
                | ScriptCapabilities::AIM_WEAPON
                | ScriptCapabilities::AIM_FROM_WEAPON
                | ScriptCapabilities::BLOCK_SHOT
                | ScriptCapabilities::FIRE_WEAPON;
        }
        ScriptKind::BloodMage | ScriptKind::OrkSmall | ScriptKind::OrkBig | ScriptKind::OrkBoss => {
            bits |= ScriptCapabilities::START_MOVING
                | ScriptCapabilities::STOP_MOVING
                | ScriptCapabilities::QUERY_WEAPON
                | ScriptCapabilities::AIM_WEAPON
                | ScriptCapabilities::AIM_FROM_WEAPON
                | ScriptCapabilities::BLOCK_SHOT
                | ScriptCapabilities::FIRE_WEAPON;
        }
    }
    ScriptCapabilities::new(bits)
}

fn instance_id(unit: UnitId) -> u32 {
    0x5454_0000u32.wrapping_add(unit.0 as u32)
}

#[derive(Clone, Copy)]
struct Pieces {
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

struct TtldState {
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

fn issue_patrol_orders(ctx: &UnitCtx) {
    let _ = spring::give_order_to_unit(ctx.unit(), cmd::REPEAT, &[1.0], 0, 0);
    if let Ok(position) = spring::get_unit_position(ctx.unit().0, false, false) {
        let _ = spring::give_order_to_unit(
            ctx.unit(),
            cmd::FIGHT,
            &[position.x, position.y, position.z],
            0,
            0,
        );
    }
}

fn face_from_position(unit: UnitId, x: f32, z: f32) {
    if let Ok(position) = spring::get_unit_position(unit.0, false, false) {
        face_direction(unit, x - position.x, z - position.z);
    }
}

async fn bob_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_MOVE);
    let initial_delay = {
        let mut state = state.borrow_mut();
        if state.is_ork && state.first_bob_run {
            let delay = (random_unit(&mut state.random) * 464.0).ceil() as u64;
            state.first_bob_run = false;
            Some(delay)
        } else {
            None
        }
    };
    if let Some(delay) = initial_delay {
        ctx.sleep(Duration::from_millis(delay)).await;
    }
    loop {
        let (piece, dead) = {
            let state = state.borrow();
            (state.pieces.torso, state.dead)
        };
        if dead {
            return;
        }
        for (offset, speed, millis) in BOB_STEPS {
            let (scale, time_scale) = {
                let state = state.borrow();
                (state.bob_scale, state.time_scale)
            };
            move_piece(&ctx, piece, Axis::Z, offset * scale, speed * scale);
            ctx.sleep(Duration::from_millis((millis * time_scale) as u64))
                .await;
        }
    }
}

/// One walk bob: torso offset and speed (both scaled by the bob scale) and the
/// pause after each move (scaled by the time scale), from shared.lua's Bob.
const BOB_STEPS: [(f32, f32, f32); 6] = [
    (3.0, 20.0, 100.0),
    (4.0, 10.0, 66.0),
    (3.0, 10.0, 66.0),
    (0.0, 20.0, 100.0),
    (-1.0, 10.0, 66.0),
    (0.0, 10.0, 66.0),
];

async fn attack_bob_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_ATTACK);
    let (piece, size, dead) = {
        let state = state.borrow();
        (state.pieces.torso, state.size_scale, state.dead)
    };
    if dead {
        return;
    }
    move_piece(
        &ctx,
        piece,
        Axis::Y,
        -10.0 - 10.0 * size,
        250.0 + 250.0 * size,
    );
    turn(
        &ctx,
        piece,
        Axis::Y,
        -20.0_f32.to_radians(),
        200.0_f32.to_radians(),
    );
    ctx.sleep(Duration::from_millis(100)).await;
    move_piece(&ctx, piece, Axis::Y, 0.0, 100.0 + 150.0 * size);
    turn(&ctx, piece, Axis::Y, 0.0, 120.0_f32.to_radians());
}

async fn attack_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_ATTACK);
    let (pieces, kind, dead, random) = {
        let state = state.borrow();
        (state.pieces, state.kind, state.dead, state.random)
    };
    if dead {
        return;
    }
    move_piece(&ctx, pieces.hand_right, Axis::Y, -30.0, 1000.0);
    turn(
        &ctx,
        pieces.hand_right,
        Axis::X,
        -20.0_f32.to_radians(),
        200.0_f32.to_radians(),
    );
    turn(
        &ctx,
        pieces.hand_right,
        Axis::Z,
        30.0_f32.to_radians(),
        100.0_f32.to_radians(),
    );
    if kind == Some(ScriptKind::BloodMage) {
        let mut seed = random;
        turn(
            &ctx,
            pieces.torso,
            Axis::X,
            -(5.0 + random_unit(&mut seed) * 2.0).to_radians(),
            120.0_f32.to_radians(),
        );
        move_piece(
            &ctx,
            pieces.torso,
            Axis::Y,
            -8.0 - random_unit(&mut seed) * 4.0,
            45.0 + random_unit(&mut seed) * 10.0,
        );
    }
    ctx.sleep(Duration::from_millis(100)).await;
    move_piece(&ctx, pieces.hand_right, Axis::Y, 0.0, 500.0);
    turn(&ctx, pieces.hand_right, Axis::X, 0.0, 40.0_f32.to_radians());
    turn(&ctx, pieces.hand_right, Axis::Z, 0.0, 50.0_f32.to_radians());
    if kind == Some(ScriptKind::BloodMage) {
        turn(&ctx, pieces.torso, Axis::X, 0.0, 100.0_f32.to_radians());
        move_piece(&ctx, pieces.torso, Axis::Y, 0.0, 30.0);
    }
}

async fn cast_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_ATTACK);
    let (pieces, level, tx, tz, unit, dead) = {
        let state = state.borrow();
        (
            state.pieces,
            state.cast_level,
            state.cast_tx,
            state.cast_tz,
            ctx.unit(),
            state.dead,
        )
    };
    if dead {
        return;
    }
    ctx.signal(SIG_MOVE);
    move_piece(&ctx, pieces.torso, Axis::Y, 0.0, 10.0);
    let _ = spring::clear_unit_goal(unit.0, true);
    if level > 1 {
        face_from_position(unit, tx, tz);
    }
    move_piece(
        &ctx,
        pieces.hand_right,
        Axis::X,
        -50.0,
        if level == 3 { 130.0 } else { 200.0 },
    );
    move_piece(
        &ctx,
        pieces.hand_right,
        Axis::Z,
        40.0,
        if level == 3 { 110.0 } else { 150.0 },
    );
    move_piece(
        &ctx,
        pieces.hand_left,
        Axis::X,
        50.0,
        if level == 3 { 130.0 } else { 200.0 },
    );
    move_piece(
        &ctx,
        pieces.hand_left,
        Axis::Z,
        40.0,
        if level == 3 { 110.0 } else { 150.0 },
    );
    turn(
        &ctx,
        pieces.hand_right,
        Axis::Z,
        25.0_f32.to_radians(),
        if level == 3 {
            120.0_f32.to_radians()
        } else {
            160.0_f32.to_radians()
        },
    );
    turn(
        &ctx,
        pieces.hand_right,
        Axis::X,
        -82.0_f32.to_radians(),
        if level == 3 {
            200.0_f32.to_radians()
        } else {
            260.0_f32.to_radians()
        },
    );
    turn(
        &ctx,
        pieces.hand_left,
        Axis::Z,
        50.0_f32.to_radians(),
        if level == 3 {
            120.0_f32.to_radians()
        } else {
            160.0_f32.to_radians()
        },
    );
    if level == 1 {
        ctx.sleep(Duration::from_millis(300)).await;
        ctx.sleep(Duration::from_millis(100)).await;
    } else if level == 2 {
        move_piece(&ctx, pieces.torso, Axis::Z, 25.0, 100.0);
        ctx.sleep(Duration::from_millis(300)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 30.0, 30.0);
        ctx.sleep(Duration::from_millis(60)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 0.0, 20.0);
        ctx.sleep(Duration::from_millis(60)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 0.0, 100.0);
    } else {
        move_piece(&ctx, pieces.torso, Axis::Z, 40.0, 120.0);
        ctx.sleep(Duration::from_millis(450)).await;
        ctx.sleep(Duration::from_millis(100)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 0.0, 80.0);
    }
    move_piece(&ctx, pieces.hand_right, Axis::X, 0.0, 100.0);
    move_piece(&ctx, pieces.hand_right, Axis::Z, 0.0, 80.0);
    move_piece(&ctx, pieces.hand_left, Axis::X, 0.0, 100.0);
    move_piece(&ctx, pieces.hand_left, Axis::Z, 0.0, 80.0);
    turn(
        &ctx,
        pieces.hand_right,
        Axis::Z,
        0.0,
        120.0_f32.to_radians(),
    );
    turn(
        &ctx,
        pieces.hand_right,
        Axis::X,
        0.0,
        200.0_f32.to_radians(),
    );
    turn(&ctx, pieces.hand_left, Axis::Z, 0.0, 120.0_f32.to_radians());
}

async fn buff_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_BUFF);
    let duration = state.borrow().cast_level.max(0) as u64;
    {
        let mut state = state.borrow_mut();
        state.bob_scale = state.static_bob_scale * 2.0;
        state.time_scale /= 2.0;
    }
    ctx.sleep(Duration::from_millis(duration.saturating_mul(1000) / 30))
        .await;
    let mut state = state.borrow_mut();
    state.bob_scale = state.static_bob_scale;
    state.time_scale = 1.0;
}

async fn stun_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_STUN);
    let (piece, duration, direction, spin_speed) = {
        let mut state = state.borrow_mut();
        let direction = if random_unit(&mut state.random) < 0.5 {
            -1.0
        } else {
            1.0
        };
        let spin_speed = 250.0 + random_unit(&mut state.random) * 100.0;
        state.bob_scale = state.static_bob_scale * 0.1;
        (
            state.pieces.torso,
            state.cast_level.max(0) as u64,
            direction,
            spin_speed,
        )
    };
    spin(
        &ctx,
        piece,
        Axis::Z,
        direction * spin_speed.to_radians(),
        400.0_f32.to_radians(),
    );
    turn(
        &ctx,
        piece,
        Axis::Y,
        90.0_f32.to_radians(),
        90.0_f32.to_radians(),
    );
    ctx.sleep(Duration::from_millis(duration.saturating_mul(1000) / 30))
        .await;
    stop_spin(&ctx, piece, Axis::Z, 400.0_f32.to_radians());
    turn(&ctx, piece, Axis::Y, 0.0, 180.0_f32.to_radians());
    let static_bob_scale = state.borrow().static_bob_scale;
    state.borrow_mut().bob_scale = static_bob_scale;
    ctx.sleep(Duration::from_millis(1000)).await;
    turn(&ctx, piece, Axis::Z, 0.0, 400.0_f32.to_radians());
}

async fn death_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_DEATH);
    let (piece, scale, size) = {
        let state = state.borrow();
        (state.pieces.torso, state.bob_scale, state.size_scale)
    };
    turn(
        &ctx,
        piece,
        Axis::Y,
        90.0_f32.to_radians(),
        250.0_f32.to_radians(),
    );
    move_piece(&ctx, piece, Axis::Z, 2.0 * scale, 20.0 * scale);
    ctx.sleep(Duration::from_millis(200)).await;
    turn(
        &ctx,
        piece,
        Axis::Z,
        90.0_f32.to_radians(),
        180.0_f32.to_radians(),
    );
    move_piece(&ctx, piece, Axis::Z, -50.0 * size, 30.0 * size);
    ctx.sleep(Duration::from_millis(1500)).await;
    state.borrow_mut().death_pending = false;
    ctx.set_killed_finished(WreckLevel::NoWreck);
}

async fn house_death_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_DEATH);
    let piece = state.borrow().pieces.house;
    play_unit_sound(ctx.unit(), "sounds/house_fall.wav", 10.0);
    move_piece(&ctx, piece, Axis::Z, -250.0, 250.0);
    let mut seed = state.borrow().random;
    turn(
        &ctx,
        piece,
        Axis::X,
        (random_unit(&mut seed) - 0.5) / 2.0,
        random_unit(&mut seed) / 2.0,
    );
    turn(
        &ctx,
        piece,
        Axis::Y,
        (random_unit(&mut seed) - 0.5) / 2.0,
        random_unit(&mut seed) / 2.0,
    );
    turn(
        &ctx,
        piece,
        Axis::Z,
        (random_unit(&mut seed) - 0.5) / 4.0,
        random_unit(&mut seed) / 4.0,
    );
    ctx.sleep(Duration::from_millis(1200)).await;
    state.borrow_mut().death_pending = false;
    ctx.set_killed_finished(WreckLevel::NoWreck);
}

async fn boss_position_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    loop {
        if state.borrow().dead {
            return;
        }
        if let Ok(position) = spring::get_unit_position(ctx.unit().0, false, false) {
            engine::set_game_number("boss_x", position.x);
            engine::set_game_number("boss_z", position.z);
        }
        ctx.sleep(Duration::from_millis(2000)).await;
    }
}

fn coward_block(ctx: &UnitCtx, target: UnitId) -> bool {
    if target.0 < 0 {
        return true;
    }
    let (Some(target_position), Ok(unit_position)) = (
        spring::get_unit_position(target.0, false, false).ok(),
        spring::get_unit_position(ctx.unit().0, false, false),
    ) else {
        return true;
    };
    let dx = target_position.x - unit_position.x;
    let dz = target_position.z - unit_position.z;
    let mut seed = ctx.unit().0 as u32 ^ engine::frame() as u32;
    let cx = unit_position.x - dx + random_unit(&mut seed) * 50.0 - 25.0;
    let cz = unit_position.z - dz + random_unit(&mut seed) * 50.0 - 25.0;
    let cy = spring::get_ground_height(cx, cz).unwrap_or(unit_position.y);
    let _ = spring::give_order_to_unit(ctx.unit(), cmd::MOVE, &[cx, cy, cz], 0, 0);
    let _ = spring::set_unit_weapon_state(
        ctx.unit().0,
        0,
        "reloadFrame",
        engine::frame() as f32 + 180.0,
    );
    true
}

fn ork_toss(unit: UnitId, target: i32, state: &Rc<RefCell<TtldState>>) {
    let (radius, strength, deunorkable) = {
        let state = state.borrow();
        (state.toss_radius, state.toss_strength, state.deunorkable)
    };
    if target < 0 || radius <= 0.0 {
        return;
    }
    let Some(target_position) = spring::get_unit_position(target, false, false).ok() else {
        return;
    };
    let Some(unit_position) = spring::get_unit_position(unit.0, false, false).ok() else {
        return;
    };
    let mut ids = [0i32; 256];
    let count = match spring::get_units_in_cylinder_into(
        target_position.x,
        target_position.z,
        radius,
        -1,
        &mut ids,
    ) {
        Ok(spring::BufferFill::Complete(count)) => count,
        _ => 0,
    };
    for other in ids[..count].iter().copied() {
        if other == unit.0 {
            continue;
        }
        let marked = spring::get_unit_rules_param_f32(other, "unorkable").unwrap_or(0.0) > 0.0;
        if !deunorkable && marked {
            continue;
        }
        let Some(other_position) = spring::get_unit_position(other, false, false).ok() else {
            continue;
        };
        let dx = other_position.x - unit_position.x;
        let dz = other_position.z - unit_position.z;
        let distance = (dx * dx + dz * dz).sqrt();
        if distance <= f32::EPSILON {
            continue;
        }
        let scale = if marked { 0.5 } else { 1.0 };
        let impulse = scale * strength * (1.0 - distance / (3.0 * radius));
        let _ = spring::add_unit_impulse(
            other,
            spring::Float3::new(impulse * dx / distance, impulse, impulse * dz / distance),
            0.0,
        );
    }
}

thread_local! {
    static PENDING_UNITS: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
    static PENDING_CALLS: RefCell<Vec<(i32, alloc::string::String, Vec<f32>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Queue a named unit-script call for the next Core-CUS tick.
///
/// The rules run inside this same module, so `call_unit_script` goes through
/// the engine and back into a module that is still inside a gadget callin; the
/// engine rejects that nested call (ApiError 1), so the animation never ran.
/// Calls are applied in order on the tick, like unit attach.
pub fn queue_call(unit: i32, function_name: &str, arguments: &[f32]) {
    if unit >= 0 {
        PENDING_CALLS.with(|pending| {
            pending.borrow_mut().push((
                unit,
                alloc::string::String::from(function_name),
                arguments.to_vec(),
            ));
        });
    }
}

pub fn queue_unit_created(unit: i32, def: i32) {
    if unit >= 0 {
        PENDING_UNITS.with(|pending| pending.borrow_mut().push((unit, def)));
    }
}

#[derive(Default)]
pub struct TtldCusModule {
    registry: CusRegistry<TtldUnitScript>,
    handles: BTreeMap<u32, CusHandle>,
}

impl TtldCusModule {
    fn attach_pending(&mut self) {
        let pending = PENDING_UNITS.with(|units| core::mem::take(&mut *units.borrow_mut()));
        for (unit_id, def) in pending {
            let Some(kind) = script_kind_for_def(def) else {
                continue;
            };
            let unit = UnitId(unit_id);
            let instance = instance_id(unit);
            if self.handles.contains_key(&instance) {
                continue;
            }
            // Drop units destroyed before the tick, whose id may even belong to
            // a new unit by now; retrying them would never end.
            if spring::get_unit_def_id(unit_id).map_or(true, |current| current.0 != def) {
                continue;
            }
            let Ok(map) = spring::get_unit_piece_map(unit_id) else {
                queue_unit_created(unit_id, def);
                continue;
            };
            let resolver = EnginePieces { map };
            let host = WasmCus::new(unit, instance);
            let engine = Rc::new(RefCell::new(host.engine()));
            let instance_state = CusInstance::construct(unit, &resolver, engine);
            let handle = self.registry.attach(instance_state);
            let _ = self.registry.with(handle, |instance| {
                instance.with_state(|script| script.set_kind(kind));
            });
            self.handles.insert(instance, handle);
            if WasmCus::attach(unit, instance, capabilities(kind)).is_err() {
                self.handles.remove(&instance);
                let _ = self.registry.detach(handle);
            }
        }
    }

    fn apply_pending_calls(&mut self) {
        let pending = PENDING_CALLS.with(|calls| core::mem::take(&mut *calls.borrow_mut()));
        for (unit_id, function_name, arguments) in pending {
            let Some(handle) = self.handles.get(&instance_id(UnitId(unit_id))).copied() else {
                continue;
            };
            self.sync_instance_frame(handle);
            let _ = self.registry.with(handle, |instance| {
                let ctx = instance.context();
                instance.with_state(|script| script.named_call(&ctx, &function_name, &arguments))
            });
        }
    }

    fn sync_instance_frame(&mut self, handle: CusHandle) {
        let frame = engine::frame() as u64;
        let _ = self.registry.with(handle, |instance| instance.tick(frame));
    }

    #[cfg(feature = "pose-probe")]
    fn run_pose_trigger(&mut self, level: i32) {
        let Some(target_def) = spring::get_unit_def_id_by_name("bloodmage").ok() else {
            return;
        };
        let Ok(units) = spring::get_all_units() else {
            return;
        };
        let Some(unit) = units.into_iter().find(|unit| {
            spring::get_unit_def_id(*unit)
                .map(|def| def.0 == target_def)
                .unwrap_or(false)
        }) else {
            return;
        };
        let instance = instance_id(UnitId(unit));
        let Some(handle) = self.handles.get(&instance).copied() else {
            return;
        };
        self.sync_instance_frame(handle);
        let arguments = if level == 1 {
            [level as f32, 0.0, 0.0]
        } else {
            [level as f32, 6000.0, 6000.0]
        };
        let mut handled = false;
        let _ = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| {
                handled = script.named_call(&ctx, "CastAnimation", &arguments);
            });
        });
        let _ = spring::echo(
            &alloc::format!(
                "[UnitScriptProbe] cus trigger unit={unit} level={level} handled={handled}"
            ),
            "",
        );
    }
}

impl CoreCusModule for TtldCusModule {
    fn cus_invoke(
        &mut self,
        instance_id: u32,
        call: u32,
        float_arguments: &[f32],
        integer_arguments: &[i32],
        result: &mut CoreCusCallResult<'_>,
    ) -> bool {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return false;
        };
        let Some(call) = UnitScriptCall::from_u32(call) else {
            return false;
        };
        if matches!(
            call,
            UnitScriptCall::RawCall | UnitScriptCall::StartMoving | UnitScriptCall::StopMoving
        ) {
            self.sync_instance_frame(handle);
        }
        let mut call_result = spring::cus::UnitScriptCallResult::default();
        let handled = self
            .registry
            .with(handle, |instance| {
                instance.invoke(call, float_arguments, integer_arguments, &mut call_result)
            })
            .unwrap_or(false);
        if !handled {
            return false;
        }
        result.int_value = call_result.int_value;
        result.float_value = call_result.float_value;
        result.bool_value = call_result.bool_value;
        result.complete = call_result.complete;
        result.int_count = call_result.int_values.len().min(result.int_values.len());
        result.int_values[..result.int_count]
            .copy_from_slice(&call_result.int_values[..result.int_count]);
        true
    }

    fn cus_call_named(
        &mut self,
        instance_id: u32,
        function_name: &str,
        arguments: &[f32],
        _return_values: &mut [f32],
        found: &mut bool,
    ) -> Option<usize> {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return Some(0);
        };
        self.sync_instance_frame(handle);
        let handled = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| script.named_call(&ctx, function_name, arguments))
        })?;
        if handled {
            *found = true;
            Some(0)
        } else {
            None
        }
    }

    fn cus_tick(&mut self, frame: u32) {
        self.attach_pending();
        self.apply_pending_calls();
        #[cfg(feature = "pose-probe")]
        {
            if frame == 30 {
                self.run_pose_trigger(1);
            }
            if frame == 90 {
                self.run_pose_trigger(2);
            }
            if frame == 145 {
                self.run_pose_trigger(3);
            }
        }
        self.registry.tick(frame as u64);
    }

    fn cus_detach(&mut self, instance_id: u32) {
        if let Some(handle) = self.handles.remove(&instance_id) {
            let _ = self.registry.detach(handle);
        }
    }
}
