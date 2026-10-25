use alloc::vec::Vec;

use spring::Float3;
use spring::consts::GAME_SPEED;

use crate::combat;
use crate::defs::{UnitKind, defs};
use crate::fx::{play_sound, play_sound_at, spawn_effect, spawn_effect_at};
use crate::random;
use crate::state::Global;
use crate::unit_scripts::ScriptCall;

use super::drain::{drain, drain_for_transfusion};
use crate::units::{self, ENEMY_TEAM, PLAYER_TEAM};
use crate::units::{def_of, health, health_value, units_in_cylinder, valid};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    spell: Spell,
    tx: f32,
    tz: f32,
    frame: i32,
) -> Option<CastOutcome> {
    let drained = match spell {
        Spell::Transfusion => drain_for_transfusion(spring, global, unit_id)?,
        _ => {
            if !drain(spring, global, unit_id, spell.cost(), false).0 {
                return None;
            }
            None
        }
    };
    let level = spell.animation_level();
    crate::unit_scripts::call(spring, unit_id, ScriptCall::CastAnimation { level, tx, tz });
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

pub fn resolve_pending(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    frame: i32,
) {
    let due = take_due(&mut global.spells.access_mut().effects, frame, |effect| {
        effect.frame
    });
    for effect in due {
        resolve(spring, global, effect, frame);
    }
    resolve_impulses(spring, global, frame);
}

pub fn fireball_exploded(spring: &spring::Spring<'_, spring::RulesSynced>, position: Float3) {
    spawn_effect_at(spring, position, CEG_FIREBALL_IMPACT);
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
const FIREBALL_LAUNCH_HEIGHT: f32 = 120.0;
const FIREBALL_TARGET_HEIGHT: f32 = 20.0;
const FIREBALL_MIN_FLIGHT: f32 = 10.0;
const FIREBALL_ELMOS_PER_FRAME: f32 = 45.0;
const ADRENALINE_FRAMES: f32 = 320.0;
const ADRENALINE_FRAMES_SPREAD: f32 = 60.0;
const MIGRAINE_STUN_FRAMES: f32 = 400.0;
const MIGRAINE_STUN_FRAMES_SPREAD: f32 = 80.0;
const MIGRAINE_FALLOFF: f32 = 1000.0;
const MIGRAINE_WAVE_SPEED: f32 = 30.0;
const POULTICE_MIN_FLIGHT: f32 = 45.0;
const POULTICE_ELMOS_PER_FRAME: f32 = 60.0;
const POULTICE_JITTER: f32 = 20.0;
const FPS: f32 = GAME_SPEED as f32;
const DEFAULT_UNIT_GRAVITY: f32 = 115.0 / FPS / FPS;
const SPELL_VOLUME: f32 = 30.0;
const FIREBALL_VOLUME: f32 = 12.0;
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

#[derive(Default)]
pub struct Spellbook {
    effects: Vec<PendingEffect>,
    impulses: Vec<DelayedImpulse>,
}

fn take_due<T: Copy>(entries: &mut Vec<T>, frame: i32, due_frame: impl Fn(&T) -> i32) -> Vec<T> {
    let due = entries
        .iter()
        .copied()
        .filter(|entry| due_frame(entry) <= frame)
        .collect();
    entries.retain(|entry| due_frame(entry) > frame);
    due
}

fn resolve(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    effect: PendingEffect,
    frame: i32,
) {
    if !valid(spring, effect.unit_id) {
        return;
    }
    match effect.spell {
        Spell::Transfusion => resolve_transfusion(spring, effect.unit_id, effect.shortfall),
        Spell::Heartburn => resolve_heartburn(spring, effect.unit_id, effect.tx, effect.tz),
        Spell::Adrenaline => resolve_adrenaline(spring, global, effect.unit_id, frame),
        Spell::Migraine => resolve_migraine(spring, global, effect.tx, effect.tz, frame),
        Spell::Poultice => resolve_poultice(spring, global, effect.unit_id, frame),
    }
}

fn resolve_transfusion(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    shortfall: Option<f32>,
) {
    let Some(health) = health(spring, unit_id) else {
        return;
    };
    play_sound(spring, "sounds/heal_spell.wav", SPELL_VOLUME, unit_id);
    let restored = health.max - shortfall.unwrap_or(0.0);
    crate::engine::check(
        spring,
        spring
            .unit_control()
            .set_unit_health(unit_id, health_value(restored)),
        "set_unit_health",
        unit_id,
    );
    spawn_effect(spring, unit_id, CEG_HEALTH);
}

fn resolve_heartburn(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    tx: f32,
    tz: f32,
) {
    let Some(origin) = units::position(spring, unit_id) else {
        return;
    };
    let Some(weapon_def_id) = defs(spring).fireball else {
        return;
    };
    play_sound(spring, "sounds/fireballfire.wav", FIREBALL_VOLUME, unit_id);
    let ty = units::ground_height(spring, tx, tz) + FIREBALL_TARGET_HEIGHT;
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
    let projectile = spring.projectile_control().spawn_projectile(
        weapon_def_id,
        &spring::projectile_control::NativeProjectileParams {
            pos: Float3 {
                x: origin.x,
                y,
                z: origin.z,
            },
            speed,
            ttl: fly_time,
            owner: unit_id,
            team: PLAYER_TEAM,
            weapon_num: 0,
            gravity: -FIREBALL_GRAVITY,
            ceg_tag: "fireball_ball".into(),
            ..Default::default()
        },
    );
    let gravity = projectile.and_then(|projectile| {
        spring
            .projectile_control()
            .set_projectile_gravity(projectile, -FIREBALL_GRAVITY)
    });
    if let Err(error) = gravity {
        spring::log_warning!(spring, "ttld-wasm", "heartburn fireball failed: {error:?}");
    }
}

fn resolve_adrenaline(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    frame: i32,
) {
    let Some(origin) = units::position(spring, unit_id) else {
        return;
    };
    play_sound(spring, "sounds/buff_spell.wav", SPELL_VOLUME, unit_id);
    let house = defs(spring).id(UnitKind::House);
    for target in units_in_cylinder(spring, origin.x, origin.z, ADRENALINE_RADIUS, PLAYER_TEAM) {
        if def_of(spring, target) == house {
            continue;
        }
        let duration = ADRENALINE_FRAMES + random::unit_float(spring) * ADRENALINE_FRAMES_SPREAD;
        combat::adrenaline(spring, global, target, duration as i32, frame);
        spawn_effect(spring, target, CEG_ADRENALINE);
    }
    spawn_effect_at(spring, origin, CEG_ADRENALINE);
}

fn resolve_migraine(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    tx: f32,
    tz: f32,
    frame: i32,
) {
    let ty = units::ground_height(spring, tx, tz);
    play_sound_at(
        spring,
        "sounds/curse_spell.wav",
        SPELL_VOLUME,
        Float3 {
            x: tx,
            y: ty,
            z: tz,
        },
    );
    for target in units_in_cylinder(spring, tx, tz, MIGRAINE_RADIUS, ENEMY_TEAM) {
        let Some(position) = units::position(spring, target) else {
            continue;
        };
        let dx = position.x - tx;
        let dz = position.z - tz;
        let distance = (dx * dx + dz * dz).sqrt();
        let delayed = DelayedImpulse {
            unit_id: target,
            impulse: Float3::default(),
            stun: (1.0 - distance / MIGRAINE_FALLOFF)
                * (MIGRAINE_STUN_FRAMES + random::unit_float(spring) * MIGRAINE_STUN_FRAMES_SPREAD),
            frame: frame + (distance / MIGRAINE_WAVE_SPEED).ceil() as i32,
        };
        global.spells.access_mut().impulses.push(delayed);
    }
    spawn_effect_at(
        spring,
        Float3 {
            x: tx,
            y: ty,
            z: tz,
        },
        CEG_MIGRAINE,
    );
}

fn resolve_poultice(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    unit_id: i32,
    frame: i32,
) {
    let Some(origin) = units::position(spring, unit_id) else {
        return;
    };
    let unit_gravity = spring
        .game()
        .get_game_map_info()
        .map(|map| map.gravity / FPS / FPS)
        .unwrap_or(DEFAULT_UNIT_GRAVITY);
    play_sound(spring, "sounds/throw_spell.wav", SPELL_VOLUME, unit_id);
    for target in units_in_cylinder(spring, origin.x, origin.z, POULTICE_RADIUS, ENEMY_TEAM) {
        let Some(position) = units::position(spring, target) else {
            continue;
        };
        let dx = position.x - origin.x;
        let dy = position.y - origin.y;
        let dz = position.z - origin.z;
        let distance = (dx * dx + dz * dz).sqrt();
        let fly_time = POULTICE_MIN_FLIGHT
            + ((distance + random::centered(spring, POULTICE_JITTER)) / POULTICE_ELMOS_PER_FRAME)
                .floor();
        let velocity = spring
            .units_info()
            .get_unit_velocity(target)
            .unwrap_or_default();
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
        crate::unit_scripts::call(spring, target, ScriptCall::StopMoving);
    }
    spawn_effect_at(spring, origin, CEG_POULTICE);
}

fn resolve_impulses(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, frame: i32) {
    let due = take_due(&mut global.spells.access_mut().impulses, frame, |impulse| {
        impulse.frame
    });
    for entry in due {
        if !valid(spring, entry.unit_id) {
            continue;
        }
        if entry.stun > 0.0 {
            combat::stun(spring, global, entry.unit_id, entry.stun as i32, frame);
        } else {
            crate::engine::check(
                spring,
                spring
                    .unit_control()
                    .add_unit_impulse(entry.unit_id, entry.impulse, 0.0),
                "add_unit_impulse",
                entry.unit_id,
            );
        }
    }
}
