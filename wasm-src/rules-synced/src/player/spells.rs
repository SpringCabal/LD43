use alloc::vec::Vec;

use spring::types::Float3;

use crate::combat;
use crate::defs::{UnitKind, defs};
use crate::fx::{play_sound, play_sound_at, spawn_effect, spawn_effect_at};
use crate::random;
use crate::state::Global;

use super::drain::{drain, drain_for_transfusion};
use crate::units::{self, ENEMY_TEAM, PLAYER_TEAM};
use crate::units::{def_of, health, health_value, units_in_cylinder, valid};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Spell {
    Transfusion,
    Heartburn,
    Adrenaline,
    Migraine,
    Poultice,
}

pub struct CastOutcome {
    pub cast_frames: i32,
}

impl Spell {
    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            1 => Some(Self::Transfusion),
            2 => Some(Self::Heartburn),
            3 => Some(Self::Adrenaline),
            4 => Some(Self::Migraine),
            5 => Some(Self::Poultice),
            _ => None,
        }
    }

    fn cost(self) -> f32 {
        match self {
            Self::Transfusion => 0.0,
            Self::Heartburn => FIREBALL_COST,
            Self::Adrenaline => BUFF_COST,
            Self::Migraine => STUN_COST,
            Self::Poultice => THROW_COST,
        }
    }

    fn animation_level(self) -> i32 {
        match self {
            Self::Transfusion | Self::Adrenaline => 1,
            Self::Heartburn => 2,
            Self::Migraine | Self::Poultice => 3,
        }
    }

    fn cast_frames(self) -> i32 {
        match self.animation_level() {
            1 => CASTING_TIME_SHORT,
            2 => CASTING_TIME,
            _ => CASTING_TIME_LONG,
        }
    }

    fn effect_delay(self) -> i32 {
        match self.animation_level() {
            3 => LONG_CAST_DELAY,
            _ => SHORT_CAST_DELAY,
        }
    }
}

pub fn cast(
    global: &Global,
    unit_id: i32,
    spell: Spell,
    tx: f32,
    tz: f32,
    frame: i32,
) -> Option<CastOutcome> {
    let drained = match spell {
        Spell::Transfusion => drain_for_transfusion(global, unit_id)?,
        _ => {
            if !drain(global, unit_id, spell.cost(), false).0 {
                return None;
            }
            None
        }
    };
    let level = spell.animation_level();
    crate::unit_scripts::call(unit_id, "CastAnimation", &[level as f32, tx, tz]);
    global.spells.access_mut().effects.push(PendingEffect {
        unit_id,
        spell,
        tx,
        tz,
        shortfall: drained,
        frame: frame + spell.effect_delay(),
    });
    Some(CastOutcome {
        cast_frames: spell.cast_frames(),
    })
}

/// Resolves the spell effects and impulses that are due.
pub fn resolve_pending(global: &Global, frame: i32) {
    let due = take_due(&mut global.spells.access_mut().effects, frame, |effect| {
        effect.frame
    });
    for effect in due {
        resolve(global, effect, frame);
    }
    resolve_impulses(global, frame);
}

/// The Heartburn fireball went off: the engine applies its weapon def's
/// damage, impulse and crater; this adds the impact sound and flames.
pub fn fireball_exploded(position: Float3) {
    // The weapon def's `soundHit` is not a key this engine reads (it wants
    // soundHitDry/soundHitWet), so the hit sound is played here.
    play_sound_at("sounds/fireballhit.wav", position);
    spawn_effect_at(position, CEG_FIREBALL_IMPACT);
}

const FIREBALL_COST: f32 = 1350.0;
const BUFF_COST: f32 = 1350.0;
const STUN_COST: f32 = 1350.0;
const THROW_COST: f32 = 1350.0;
const CASTING_TIME_SHORT: i32 = 8;
const CASTING_TIME: i32 = 12;
const CASTING_TIME_LONG: i32 = 18;
const SHORT_CAST_DELAY: i32 = 9;
const LONG_CAST_DELAY: i32 = 14;
const ADRENALINE_RADIUS: f32 = 450.0;
const MIGRAINE_RADIUS: f32 = 450.0;
const POULTICE_RADIUS: f32 = 720.0;
const FIREBALL_GRAVITY: f32 = 0.18;
/// The fireball leaves this far above the mage and aims this far above the
/// ground at its target.
const FIREBALL_LAUNCH_HEIGHT: f32 = 120.0;
const FIREBALL_TARGET_HEIGHT: f32 = 20.0;
/// Flight time in frames: a minimum plus one frame per this many elmos.
const FIREBALL_MIN_FLIGHT: f32 = 10.0;
const FIREBALL_ELMOS_PER_FRAME: f32 = 45.0;
/// Adrenaline lasts this many frames, plus up to the spread.
const ADRENALINE_FRAMES: f32 = 320.0;
const ADRENALINE_FRAMES_SPREAD: f32 = 60.0;
/// Migraine stuns for this many frames, plus up to the spread, scaled down
/// linearly to nothing at the falloff distance. Its wave travels this many
/// elmos per frame.
const MIGRAINE_STUN_FRAMES: f32 = 400.0;
const MIGRAINE_STUN_FRAMES_SPREAD: f32 = 80.0;
const MIGRAINE_FALLOFF: f32 = 1000.0;
const MIGRAINE_WAVE_SPEED: f32 = 30.0;
/// Poultice throws for a minimum flight time plus one frame per this many
/// elmos (give or take half the jitter), and its wave travels just as fast.
const POULTICE_MIN_FLIGHT: f32 = 45.0;
const POULTICE_ELMOS_PER_FRAME: f32 = 60.0;
const POULTICE_JITTER: f32 = 20.0;
// Game.gravity is exposed to Lua as a positive downward acceleration. The
// engine's unit impulse uses +Y as up, so the launch velocity needs the
// positive value here; the ground move type applies downward gravity after
// the impulse. A negative value points Poultice into the ground.
const DEFAULT_UNIT_GRAVITY: f32 = 115.0 / 30.0 / 30.0;
const CEG_HEALTH: &str = "feature_slurp_spawner";
const CEG_MIGRAINE: &str = "migraine_pulse_spawner";
const CEG_ADRENALINE: &str = "adrenaline_sparkles";
const CEG_POULTICE: &str = "gtfo_pulse";
const CEG_FIREBALL_IMPACT: &str = "fireball_impact";

#[derive(Clone, Copy)]
struct PendingEffect {
    unit_id: i32,
    spell: Spell,
    tx: f32,
    tz: f32,
    shortfall: Option<f32>,
    frame: i32,
}

#[derive(Clone, Copy)]
struct DelayedImpulse {
    unit_id: i32,
    impulse: Float3,
    stun: f32,
    frame: i32,
}

/// Spells cast but not resolved yet.
#[derive(Default)]
pub struct Spellbook {
    /// Cast spells that take effect after their cast delay.
    effects: Vec<PendingEffect>,
    /// Migraine stuns and Poultice throws, delayed by their wave's travel.
    impulses: Vec<DelayedImpulse>,
}

/// Removes and returns the entries due at `frame`.
fn take_due<T: Copy>(entries: &mut Vec<T>, frame: i32, due_frame: impl Fn(&T) -> i32) -> Vec<T> {
    let due = entries
        .iter()
        .copied()
        .filter(|entry| due_frame(entry) <= frame)
        .collect();
    entries.retain(|entry| due_frame(entry) > frame);
    due
}

fn resolve(global: &Global, effect: PendingEffect, frame: i32) {
    if !valid(effect.unit_id) {
        return;
    }
    match effect.spell {
        Spell::Transfusion => resolve_transfusion(effect.unit_id, effect.shortfall),
        Spell::Heartburn => resolve_heartburn(effect.unit_id, effect.tx, effect.tz),
        Spell::Adrenaline => resolve_adrenaline(global, effect.unit_id, frame),
        Spell::Migraine => resolve_migraine(global, effect.tx, effect.tz, frame),
        Spell::Poultice => resolve_poultice(global, effect.unit_id, frame),
    }
}

fn resolve_transfusion(unit_id: i32, shortfall: Option<f32>) {
    let Some(health) = health(unit_id) else {
        return;
    };
    play_sound("sounds/heal_spell.wav", unit_id);
    let restored = health.max - shortfall.unwrap_or(0.0);
    crate::engine::check(
        spring::set_unit_health(unit_id, health_value(restored)),
        "set_unit_health",
        unit_id,
    );
    spawn_effect(unit_id, CEG_HEALTH);
}

/// Launches the fireball. It is the weapon def's own projectile, as in Lua:
/// it explodes where it lands or when its flight time runs out, and the
/// engine applies the weapon's area damage, impulse and crater to every team,
/// whether or not the mage is still alive by then.
fn resolve_heartburn(unit_id: i32, tx: f32, tz: f32) {
    let Some(origin) = units::position(unit_id) else {
        return;
    };
    let Some(weapon_def_id) = defs().fireball else {
        return;
    };
    play_sound("sounds/fireballfire.wav", unit_id);
    let ty = units::ground_height(tx, tz) + FIREBALL_TARGET_HEIGHT;
    let y = origin.y + FIREBALL_LAUNCH_HEIGHT;
    let dx = tx - origin.x;
    let dy = ty - y;
    let dz = tz - origin.z;
    let fly_time =
        FIREBALL_MIN_FLIGHT + ((dx * dx + dz * dz).sqrt() / FIREBALL_ELMOS_PER_FRAME).floor();
    let speed = Float3 {
        x: dx / fly_time,
        y: fly_time * FIREBALL_GRAVITY / 2.0 + dy / fly_time,
        z: dz / fly_time,
    };
    let projectile = spring::spawn_projectile(
        weapon_def_id,
        spring::ProjectileParams {
            position: Float3 {
                x: origin.x,
                y,
                z: origin.z,
            },
            speed,
            ttl: fly_time,
            owner: unit_id,
            team: PLAYER_TEAM,
            // A spell projectile, not a shot from one of the mage's weapons.
            weapon_number: -1,
            gravity: -FIREBALL_GRAVITY,
            ceg_tag: "fireball_ball",
            ..Default::default()
        },
    );
    let gravity = projectile.and_then(|projectile| {
        spring::projectile_control::set_projectile_gravity(projectile, -FIREBALL_GRAVITY)
    });
    if let Err(error) = gravity {
        spring::log_warning!("ttld-wasm", "heartburn fireball failed: {error:?}");
    }
}

fn resolve_adrenaline(global: &Global, unit_id: i32, frame: i32) {
    let Some(origin) = units::position(unit_id) else {
        return;
    };
    play_sound("sounds/buff_spell.wav", unit_id);
    let house = defs().id(UnitKind::House);
    for target in units_in_cylinder(origin.x, origin.z, ADRENALINE_RADIUS, PLAYER_TEAM) {
        if def_of(target) == house {
            continue;
        }
        let duration = ADRENALINE_FRAMES + random::unit_float() * ADRENALINE_FRAMES_SPREAD;
        combat::adrenaline(global, target, duration as i32, frame);
        spawn_effect(target, CEG_ADRENALINE);
    }
    spawn_effect_at(origin, CEG_ADRENALINE);
}

fn resolve_migraine(global: &Global, tx: f32, tz: f32, frame: i32) {
    let ty = units::ground_height(tx, tz);
    play_sound_at(
        "sounds/curse_spell.wav",
        Float3 {
            x: tx,
            y: ty,
            z: tz,
        },
    );
    for target in units_in_cylinder(tx, tz, MIGRAINE_RADIUS, ENEMY_TEAM) {
        let Some(position) = units::position(target) else {
            continue;
        };
        let dx = position.x - tx;
        let dz = position.z - tz;
        let distance = (dx * dx + dz * dz).sqrt();
        let delayed = DelayedImpulse {
            unit_id: target,
            impulse: Float3::default(),
            stun: (1.0 - distance / MIGRAINE_FALLOFF)
                * (MIGRAINE_STUN_FRAMES + random::unit_float() * MIGRAINE_STUN_FRAMES_SPREAD),
            frame: frame + (distance / MIGRAINE_WAVE_SPEED).ceil() as i32,
        };
        global.spells.access_mut().impulses.push(delayed);
    }
    spawn_effect_at(
        Float3 {
            x: tx,
            y: ty,
            z: tz,
        },
        CEG_MIGRAINE,
    );
}

fn resolve_poultice(global: &Global, unit_id: i32, frame: i32) {
    let Some(origin) = units::position(unit_id) else {
        return;
    };
    let unit_gravity = spring::game::get_game_map_info(0)
        .map(|map| map.gravity / 30.0 / 30.0)
        .unwrap_or(DEFAULT_UNIT_GRAVITY);
    play_sound("sounds/throw_spell.wav", unit_id);
    for target in units_in_cylinder(origin.x, origin.z, POULTICE_RADIUS, ENEMY_TEAM) {
        let Some(position) = units::position(target) else {
            continue;
        };
        // Poultice is a knockback: launch each enemy away from the mage.
        // Using origin - position here makes the impulse point inward and
        // turns the spell into a pull instead.
        let dx = position.x - origin.x;
        let dy = position.y - origin.y;
        let dz = position.z - origin.z;
        let distance = (dx * dx + dz * dz).sqrt();
        let fly_time = POULTICE_MIN_FLIGHT
            + ((distance + random::centered(POULTICE_JITTER)) / POULTICE_ELMOS_PER_FRAME).floor();
        let velocity = spring::units_info::get_unit_velocity(target).unwrap_or_default();
        let impulse = Float3 {
            x: dx / fly_time - velocity.x,
            y: fly_time * unit_gravity / 2.0 + dy / fly_time - velocity.y,
            z: dz / fly_time - velocity.z,
        };
        let delayed = DelayedImpulse {
            unit_id: target,
            impulse,
            stun: 0.0,
            frame: frame + (distance / POULTICE_ELMOS_PER_FRAME).ceil() as i32,
        };
        global.spells.access_mut().impulses.push(delayed);
        crate::unit_scripts::call(target, "StopMoving", &[]);
    }
    spawn_effect_at(origin, CEG_POULTICE);
}

fn resolve_impulses(global: &Global, frame: i32) {
    let due = take_due(&mut global.spells.access_mut().impulses, frame, |impulse| {
        impulse.frame
    });
    for entry in due {
        if !valid(entry.unit_id) {
            continue;
        }
        if entry.stun > 0.0 {
            combat::stun(global, entry.unit_id, entry.stun as i32, frame);
        } else {
            crate::engine::check(
                spring::add_unit_impulse(entry.unit_id, entry.impulse, 0.0),
                "add_unit_impulse",
                entry.unit_id,
            );
        }
    }
}
