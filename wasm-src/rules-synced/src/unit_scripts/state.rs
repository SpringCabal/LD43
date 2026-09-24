//! The per-unit state a unit script shares with its tasks.

use spring::cus::{InitCtx, Piece};

use super::kinds::ScriptKind;

#[derive(Clone, Copy)]
pub(crate) struct Pieces {
    pub(super) hand_right: Piece,
    pub(super) hand_left: Piece,
    pub(super) torso: Piece,
    pub(super) house: Piece,
}

impl Pieces {
    pub(super) fn from_init(ctx: &InitCtx<'_>) -> Self {
        Self {
            hand_right: ctx.piece("Hand_Right"),
            hand_left: ctx.piece("Hand_Left"),
            torso: ctx.piece("Torso"),
            house: ctx.piece("House"),
        }
    }
}

pub(crate) struct TtldState {
    pub(super) kind: Option<ScriptKind>,
    pub(super) pieces: Pieces,
    pub(super) dead: bool,
    pub(super) death_pending: bool,
    pub(super) first_bob_run: bool,
    pub(super) bob_scale: f32,
    pub(super) static_bob_scale: f32,
    pub(super) time_scale: f32,
    pub(super) size_scale: f32,
    pub(super) is_ork: bool,
    pub(super) death_sound: Option<&'static str>,
    pub(super) death_sound_chance: f32,
    pub(super) toss_radius: f32,
    pub(super) toss_strength: f32,
    pub(super) deunorkable: bool,
    pub(super) cast_level: i32,
    pub(super) cast_tx: f32,
    pub(super) cast_tz: f32,
    /// How long the current buff or stun animation lasts, in frames.
    pub(super) effect_frames: i32,
    /// The blood mage's ground target while the player attacks, Lua's
    /// `GG.fireTx`/`GG.fireTz`: without one it holds its fire.
    pub(super) fire_target: Option<(f32, f32)>,
}

impl TtldState {
    pub(super) fn new(ctx: &mut InitCtx<'_>) -> Self {
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
            effect_frames: 0,
            fire_target: None,
        }
    }
}
