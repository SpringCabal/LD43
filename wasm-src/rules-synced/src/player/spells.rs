use alloc::vec::Vec;

use spring::types::Float3;

use crate::combat;
use crate::defs::{UnitKind, defs};
use crate::engine::set_game_number;
use crate::fx::{play_sound, play_sound_at, spawn_effect, spawn_effect_at};
use crate::random;

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

pub fn cast(unit_id: i32, spell: Spell, tx: f32, tz: f32, frame: i32) -> Option<CastOutcome> {
    let drained = match spell {
        Spell::Transfusion => drain_for_transfusion(unit_id)?,
        _ => {
            if !drain(unit_id, spell.cost(), false).0 {
                return None;
            }
            None
        }
    };
    let level = spell.animation_level();
    crate::unit_scripts::queue_call(unit_id, "CastAnimation", &[level as f32, tx, tz]);
    publish_cast(frame, spell.cast_frames());
    schedule(PendingEffect {
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

pub fn resolve_pending(frame: i32) {
    let due = with_pending(|pending| {
        let due: Vec<PendingEffect> = pending
            .iter()
            .copied()
            .filter(|effect| effect.frame <= frame)
            .collect();
        pending.retain(|effect| effect.frame > frame);
        due
    })
    .unwrap_or_default();
    for effect in due {
        resolve(effect, frame);
    }
    resolve_impulses(frame);
    resolve_fireball_impacts(frame);
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
const FIREBALL_RADIUS: f32 = 250.0;
const FIREBALL_DAMAGE: f32 = 1600.0;
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

#[derive(Clone, Copy)]
struct FireballImpact {
    unit_id: i32,
    x: f32,
    z: f32,
    frame: i32,
}

thread_local! {
    static PENDING: core::cell::RefCell<Vec<PendingEffect>> = const {
        core::cell::RefCell::new(Vec::new())
    };
    static IMPULSES: core::cell::RefCell<Vec<DelayedImpulse>> = const {
        core::cell::RefCell::new(Vec::new())
    };
    static FIREBALL_IMPACTS: core::cell::RefCell<Vec<FireballImpact>> = const {
        core::cell::RefCell::new(Vec::new())
    };
}

fn with_pending<R>(function: impl FnOnce(&mut Vec<PendingEffect>) -> R) -> Option<R> {
    PENDING.with(|pending| pending.try_borrow_mut().ok().map(|mut p| function(&mut p)))
}

fn with_impulses<R>(function: impl FnOnce(&mut Vec<DelayedImpulse>) -> R) -> Option<R> {
    IMPULSES.with(|impulses| impulses.try_borrow_mut().ok().map(|mut i| function(&mut i)))
}

fn with_fireball_impacts<R>(function: impl FnOnce(&mut Vec<FireballImpact>) -> R) -> Option<R> {
    FIREBALL_IMPACTS.with(|impacts| impacts.try_borrow_mut().ok().map(|mut i| function(&mut i)))
}

fn publish_cast(frame: i32, cast_frames: i32) {
    set_game_number("castingFreeze", (frame + cast_frames) as f32);
    set_game_number("castAnimTime", (cast_frames + 8) as f32);
    set_game_number("start_cast", frame as f32);
}

fn schedule(effect: PendingEffect) {
    let _ = with_pending(|pending| pending.push(effect));
}

fn resolve(effect: PendingEffect, frame: i32) {
    if !valid(effect.unit_id) {
        return;
    }
    match effect.spell {
        Spell::Transfusion => resolve_transfusion(effect.unit_id, effect.shortfall),
        Spell::Heartburn => resolve_heartburn(effect.unit_id, effect.tx, effect.tz, frame),
        Spell::Adrenaline => resolve_adrenaline(effect.unit_id, frame),
        Spell::Migraine => resolve_migraine(effect.tx, effect.tz, frame),
        Spell::Poultice => resolve_poultice(effect.unit_id, frame),
    }
}

fn resolve_transfusion(unit_id: i32, shortfall: Option<f32>) {
    let Some(health) = health(unit_id) else {
        return;
    };
    play_sound("sounds/heal_spell.wav", unit_id);
    let restored = health.max - shortfall.unwrap_or(0.0);
    let _ = spring::set_unit_health(unit_id, health_value(restored));
    spawn_effect(unit_id, CEG_HEALTH);
}

fn resolve_heartburn(unit_id: i32, tx: f32, tz: f32, frame: i32) {
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
    if let Ok(projectile) = projectile {
        let _ = spring::projectile_control::set_projectile_gravity(projectile, -FIREBALL_GRAVITY);
        // The visual projectile is deliberately damage-free.  Its TTL is not
        // a reliable impact callback in every WASM renderer, so the synced
        // impact below applies the spell damage at the requested destination
        // exactly once.
        let _ = spring::projectile_control::set_projectile_damages(projectile, 0, "default", 0.0);
    }
    let _ = with_fireball_impacts(|impacts| {
        impacts.push(FireballImpact {
            unit_id,
            x: tx,
            z: tz,
            frame: frame + fly_time as i32,
        })
    });
}

fn resolve_adrenaline(unit_id: i32, frame: i32) {
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
        combat::adrenaline(target, duration as i32, frame);
        spawn_effect(target, CEG_ADRENALINE);
    }
    spawn_effect_at(origin, CEG_ADRENALINE);
}

fn resolve_migraine(tx: f32, tz: f32, frame: i32) {
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
        let _ = with_impulses(|impulses| {
            impulses.push(DelayedImpulse {
                unit_id: target,
                impulse: Float3::default(),
                stun: (1.0 - distance / MIGRAINE_FALLOFF)
                    * (MIGRAINE_STUN_FRAMES + random::unit_float() * MIGRAINE_STUN_FRAMES_SPREAD),
                frame: frame + (distance / MIGRAINE_WAVE_SPEED).ceil() as i32,
            })
        });
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

fn resolve_poultice(unit_id: i32, frame: i32) {
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
        let _ = with_impulses(|impulses| {
            impulses.push(DelayedImpulse {
                unit_id: target,
                impulse,
                stun: 0.0,
                frame: frame + (distance / POULTICE_ELMOS_PER_FRAME).ceil() as i32,
            })
        });
        crate::unit_scripts::queue_call(target, "StopMoving", &[]);
    }
    spawn_effect_at(origin, CEG_POULTICE);
}

fn resolve_impulses(frame: i32) {
    let due = with_impulses(|impulses| {
        let due: Vec<DelayedImpulse> = impulses
            .iter()
            .copied()
            .filter(|impulse| impulse.frame <= frame)
            .collect();
        impulses.retain(|impulse| impulse.frame > frame);
        due
    })
    .unwrap_or_default();
    for entry in due {
        if !valid(entry.unit_id) {
            continue;
        }
        if entry.stun > 0.0 {
            combat::stun(entry.unit_id, entry.stun as i32, frame);
        } else {
            let _ = spring::add_unit_impulse(entry.unit_id, entry.impulse, 0.0);
        }
    }
}

fn resolve_fireball_impacts(frame: i32) {
    let due = with_fireball_impacts(|impacts| {
        let due: Vec<FireballImpact> = impacts
            .iter()
            .copied()
            .filter(|impact| impact.frame <= frame)
            .collect();
        impacts.retain(|impact| impact.frame > frame);
        due
    })
    .unwrap_or_default();

    let Some(weapon_def_id) = defs().fireball else {
        return;
    };
    for impact in due {
        if !valid(impact.unit_id) {
            continue;
        }
        let y = units::ground_height(impact.x, impact.z) + FIREBALL_TARGET_HEIGHT;
        let position = Float3 {
            x: impact.x,
            y,
            z: impact.z,
        };
        play_sound_at("sounds/fireballhit.wav", position);
        spawn_effect_at(position, CEG_FIREBALL_IMPACT);
        for target in units_in_cylinder(impact.x, impact.z, FIREBALL_RADIUS, ENEMY_TEAM) {
            let _ = spring::add_unit_damage(
                target,
                FIREBALL_DAMAGE,
                0.0,
                weapon_def_id,
                impact.unit_id,
                Float3::default(),
            );
        }
    }
}
