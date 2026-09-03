use alloc::vec::Vec;

use spring::types::Float3;

use crate::combat;
use crate::random;
use crate::units::{self, ENEMY_TEAM, NEUTRAL_TEAM, PLAYER_TEAM};

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
    let _ = spring::typed::call_unit_script(unit_id, "CastAnimation", &[level as f32, tx, tz], 0);
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
}

const BLOOD_MAGIC_RANGE: f32 = 600.0;
const FIREBALL_COST: f32 = 1350.0;
const BUFF_COST: f32 = 1350.0;
const STUN_COST: f32 = 1350.0;
const THROW_COST: f32 = 1350.0;
const LIFE_BONUS: f32 = 500.0;
const CASTING_TIME_SHORT: i32 = 8;
const CASTING_TIME: i32 = 12;
const CASTING_TIME_LONG: i32 = 18;
const SHORT_CAST_DELAY: i32 = 9;
const LONG_CAST_DELAY: i32 = 14;
const DRAIN_SEARCH_LIMIT: i32 = 50;
const ADRENALINE_RADIUS: f32 = 450.0;
const MIGRAINE_RADIUS: f32 = 450.0;
const POULTICE_RADIUS: f32 = 720.0;
const FIREBALL_GRAVITY: f32 = 0.18;
const UNIT_GRAVITY: f32 = -120.0 / 30.0 / 30.0;
const CEG_SINGLE_BLOOD: &str = "feature_poof_single";
const CEG_FLAMES: &str = "feature_poof_spawner";
const CEG_HEALTH: &str = "feature_slurp_spawner";
const CEG_MIGRAINE: &str = "migraine_pulse_spawner";
const CEG_ADRENALINE: &str = "adrenaline_sparkles";
const CEG_POULTICE: &str = "gtfo_pulse";
const SOUND_DRAIN: &str = "sounds/blooddrain.wav";
// LuaRulesParams::RULESPARAMLOS_PUBLIC
const LOS_PUBLIC: i32 = 32;

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

thread_local! {
    static PENDING: core::cell::RefCell<Vec<PendingEffect>> = const {
        core::cell::RefCell::new(Vec::new())
    };
    static IMPULSES: core::cell::RefCell<Vec<DelayedImpulse>> = const {
        core::cell::RefCell::new(Vec::new())
    };
}

fn with_pending<R>(function: impl FnOnce(&mut Vec<PendingEffect>) -> R) -> Option<R> {
    PENDING.with(|pending| pending.try_borrow_mut().ok().map(|mut p| function(&mut p)))
}

fn with_impulses<R>(function: impl FnOnce(&mut Vec<DelayedImpulse>) -> R) -> Option<R> {
    IMPULSES.with(|impulses| impulses.try_borrow_mut().ok().map(|mut i| function(&mut i)))
}

fn publish_cast(frame: i32, cast_frames: i32) {
    set_number("castingFreeze", (frame + cast_frames) as f32);
    set_number("castAnimTime", (cast_frames + 8) as f32);
    set_number("start_cast", frame as f32);
}

fn set_number(name: &str, value: f32) {
    let value = spring::generated::owned::rules_params::RulesParamValue {
        type_: spring::generated::owned::rules_params::RulesParamType::RulesparamTypeFloat,
        bool_value: false,
        float_value: value,
        string_value: alloc::string::String::new(),
    };
    let _ = spring::generated::owned::rules_params::set_game_rules_param(name, &value, LOS_PUBLIC);
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
        Spell::Heartburn => resolve_heartburn(effect.unit_id, effect.tx, effect.tz),
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

fn resolve_heartburn(unit_id: i32, tx: f32, tz: f32) {
    let Some(origin) = units::position(unit_id) else {
        return;
    };
    let Some(weapon_def_id) = spring::weapon_defs::get_weapon_def_id("fireball").ok() else {
        return;
    };
    play_sound("sounds/fireballfire.wav", unit_id);
    let ty = units::ground_height(tx, tz) + 20.0;
    let y = origin.y + 120.0;
    let dx = tx - origin.x;
    let dy = ty - y;
    let dz = tz - origin.z;
    let fly_time = 10.0 + ((dx * dx + dz * dz).sqrt() / 45.0).floor();
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
            gravity: -FIREBALL_GRAVITY,
            ..Default::default()
        },
    );
    if let Ok(projectile) = projectile {
        let _ = spring::projectile_control::set_projectile_gravity(projectile, -FIREBALL_GRAVITY);
    }
}

fn resolve_adrenaline(unit_id: i32, frame: i32) {
    let Some(origin) = units::position(unit_id) else {
        return;
    };
    play_sound("sounds/buff_spell.wav", unit_id);
    let house = units::def_id(units::HOUSE);
    for target in units_in_cylinder(origin.x, origin.z, ADRENALINE_RADIUS, PLAYER_TEAM) {
        if def_of(target) == house {
            continue;
        }
        combat::adrenaline(target, (320.0 + random::unit_float() * 60.0) as i32, frame);
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
                stun: (1.0 - distance / 1000.0) * (400.0 + random::unit_float() * 80.0),
                frame: frame + (distance / 30.0).ceil() as i32,
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
    play_sound("sounds/throw_spell.wav", unit_id);
    for target in units_in_cylinder(origin.x, origin.z, POULTICE_RADIUS, ENEMY_TEAM) {
        let Some(position) = units::position(target) else {
            continue;
        };
        let dx = origin.x - position.x;
        let dy = origin.y - position.y;
        let dz = origin.z - position.z;
        let distance = (dx * dx + dz * dz).sqrt();
        let fly_time = 45.0 + ((distance + random::centered(20.0)) / 60.0).floor();
        let velocity = spring::units_info::get_unit_velocity(target).unwrap_or_default();
        let impulse = Float3 {
            x: dx / fly_time - velocity.x,
            y: fly_time * UNIT_GRAVITY / 2.0 + dy / fly_time - velocity.y,
            z: dz / fly_time - velocity.z,
        };
        let _ = with_impulses(|impulses| {
            impulses.push(DelayedImpulse {
                unit_id: target,
                impulse,
                stun: 0.0,
                frame: frame + (distance / 60.0).ceil() as i32,
            })
        });
        let _ = spring::typed::call_unit_script(target, "StopMoving", &[], 0);
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

fn drain_for_transfusion(unit_id: i32) -> Option<Option<f32>> {
    let health = health(unit_id)?;
    if health.current >= health.max {
        return None;
    }
    let required = LIFE_BONUS + health.max - health.current;
    let (found, shortfall) = drain(unit_id, required, true);
    if !found && shortfall.is_none() {
        return None;
    }
    Some(shortfall.map(|value| value - LIFE_BONUS))
}

fn drain(collector_id: i32, required: f32, allow_partial: bool) -> (bool, Option<f32>) {
    let Some(origin) = units::position(collector_id) else {
        return (false, None);
    };
    let house = units::def_id(units::HOUSE);
    let mut remaining = required;
    let mut to_kill: Vec<i32> = Vec::new();
    let mut to_restore: Vec<i32> = Vec::new();
    for _ in 0..DRAIN_SEARCH_LIMIT {
        let Ok(target) = spring::get_unit_nearest_ally(collector_id, BLOOD_MAGIC_RANGE) else {
            break;
        };
        let target = target.0;
        if target < 0 {
            break;
        }
        if def_of(target) == house {
            let _ = spring::transfer_unit(target, NEUTRAL_TEAM, false, false);
            to_restore.push(target);
            continue;
        }
        let Some(health) = health(target) else {
            break;
        };
        let available = health.current + LIFE_BONUS;
        if available > remaining {
            spawn_effect(target, CEG_FLAMES);
            play_sound_at(SOUND_DRAIN, origin);
            blood_trail(origin, target);
            let _ = spring::set_unit_health(target, health_value(available - remaining));
            restore(&to_restore);
            record_drained(consume(&to_kill, origin));
            return (true, None);
        }
        to_kill.push(target);
        let _ = spring::transfer_unit(target, NEUTRAL_TEAM, false, false);
        remaining -= available;
    }
    restore(&to_restore);
    if allow_partial && !to_kill.is_empty() {
        record_drained(consume(&to_kill, origin));
        return (false, Some(remaining));
    }
    restore(&to_kill);
    (false, None)
}

fn consume(units: &[i32], origin: Float3) -> i32 {
    let mut drained = 0;
    for unit_id in units {
        spawn_effect(*unit_id, CEG_FLAMES);
        play_sound_at(SOUND_DRAIN, origin);
        blood_trail(origin, *unit_id);
        let _ = spring::destroy_unit(
            *unit_id,
            spring::generated::owned::unit_control::DestroyUnitOptions {
                selfd: true,
                reclaimed: false,
                attacker_id: -1,
                recycle_id: false,
            },
        );
        drained += 1;
    }
    drained
}

fn record_drained(count: i32) {
    let _ = crate::state::with_world(|world| world.allies_drained += count);
}

fn restore(units: &[i32]) {
    for unit_id in units {
        let _ = spring::transfer_unit(*unit_id, PLAYER_TEAM, false, false);
    }
}

fn blood_trail(origin: Float3, target_id: i32) {
    let Some(target) = units::position(target_id) else {
        return;
    };
    let dx = origin.x - target.x;
    let dy = origin.y - target.y;
    let dz = origin.z - target.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
    let step = Float3 {
        x: dx / distance,
        y: dy / distance,
        z: dz / distance,
    };
    let mut travelled = 10.0;
    while travelled < distance {
        spawn_effect_at(
            Float3 {
                x: target.x + step.x * travelled,
                y: target.y + step.y * travelled,
                z: target.z + step.z * travelled,
            },
            CEG_SINGLE_BLOOD,
        );
        travelled += 10.0;
    }
}

fn units_in_cylinder(x: f32, z: f32, radius: f32, allegiance: i32) -> Vec<i32> {
    let mut output = alloc::vec![0i32; 512];
    match spring::get_units_in_cylinder_into(x, z, radius, allegiance, &mut output) {
        Ok(spring::BufferFill::Complete(count)) => {
            output.truncate(count);
            output
        }
        _ => Vec::new(),
    }
}

fn spawn_effect(unit_id: i32, ceg_name: &str) {
    let Some(position) = units::position(unit_id) else {
        return;
    };
    spawn_effect_at(position, ceg_name);
}

fn spawn_effect_at(position: Float3, ceg_name: &str) {
    let _ = spring::spawn_ceg(ceg_name, position, Float3::default(), 20.0, 20.0, 0.0);
}

fn play_sound(name: &str, unit_id: i32) {
    let Some(position) = units::position(unit_id) else {
        return;
    };
    play_sound_at(name, position);
}

fn play_sound_at(name: &str, position: Float3) {
    let _ = spring::sound::play_sound_file(name, 1.0, position, Float3::default(), 0);
}

fn valid(unit_id: i32) -> bool {
    spring::valid_unit_id(unit_id).unwrap_or(false)
}

fn def_of(unit_id: i32) -> Option<i32> {
    spring::units_info::get_unit_def_id(unit_id).ok()
}

fn health(unit_id: i32) -> Option<Health> {
    let health = spring::units_info::get_unit_health(unit_id).ok()?;
    Some(Health {
        current: health.health,
        max: health.max_health,
    })
}

struct Health {
    current: f32,
    max: f32,
}

fn health_value(health: f32) -> spring::types::UnitHealthValue {
    spring::types::UnitHealthValue {
        health,
        capture: -1.0,
        paralyze: -1.0,
        build: -1.0,
        use_amounts: false,
    }
}
