use alloc::collections::BTreeMap;
use std::sync::Mutex;

use springcabal::defs::DefCache;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    BloodMage,
    House,
    OrkSmall,
    OrkBig,
    OrkBoss,
    Swordsman,
    Crossbowman,
    Coward,
    Peasant,
    Builder,
}

impl UnitKind {
    const ALL: [Self; 10] = [
        Self::BloodMage,
        Self::House,
        Self::OrkSmall,
        Self::OrkBig,
        Self::OrkBoss,
        Self::Swordsman,
        Self::Crossbowman,
        Self::Coward,
        Self::Peasant,
        Self::Builder,
    ];

    pub fn def_name(self) -> &'static str {
        match self {
            Self::BloodMage => "bloodmage",
            Self::House => "house",
            Self::OrkSmall => "orksmall",
            Self::OrkBig => "orkbig",
            Self::OrkBoss => "orkboss",
            Self::Swordsman => "swordsman",
            Self::Crossbowman => "crossbowman",
            Self::Coward => "coward",
            Self::Peasant => "peasant",
            Self::Builder => "builder",
        }
    }

    pub fn is_ork(self) -> bool {
        matches!(self, Self::OrkSmall | Self::OrkBig | Self::OrkBoss)
    }
}

#[derive(Clone, Copy)]
pub struct DefInfo {
    pub max_weapon_range: f32,
    pub movement: Option<Movement>,
    pub footprint: Option<(f32, f32)>,
    pub draw_scale: Option<(f32, f32)>,
}

#[derive(Clone, Copy)]
pub struct Movement {
    pub speed: f32,
    pub reverse_speed: f32,
    pub turn_rate: f32,
    pub max_acc: f32,
    pub max_dec: f32,
}

pub struct Defs {
    units: [Option<i32>; UnitKind::ALL.len()],
    pub fireball: Option<i32>,
    info: Mutex<BTreeMap<i32, DefInfo>>,
}

static DEFS: DefCache<Defs> = DefCache::new();

pub fn defs(spring: &spring::Spring<'_, spring::RulesSynced>) -> &'static Defs {
    DEFS.get_or_init(|| Defs::resolve(spring))
}

impl Defs {
    fn resolve(spring: &spring::Spring<'_, spring::RulesSynced>) -> Self {
        Self {
            units: UnitKind::ALL.map(|kind| {
                spring
                    .unit_defs()
                    .get_unit_def_id_by_name(kind.def_name())
                    .ok()
            }),
            fireball: spring.weapon_defs().get_weapon_def_id("fireball").ok(),
            info: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn id(&self, kind: UnitKind) -> Option<i32> {
        self.units[kind as usize]
    }

    pub fn kind(&self, def_id: i32) -> Option<UnitKind> {
        UnitKind::ALL
            .into_iter()
            .find(|kind| self.id(*kind) == Some(def_id))
    }

    pub fn is(&self, def_id: i32, kind: UnitKind) -> bool {
        self.id(kind) == Some(def_id)
    }

    pub fn is_ork(&self, def_id: i32) -> bool {
        self.kind(def_id).is_some_and(UnitKind::is_ork)
    }

    pub fn info(&self, spring: &spring::Spring<'_, spring::RulesSynced>, def_id: i32) -> DefInfo {
        if let Some(info) = self.info.lock().expect("def info cache").get(&def_id) {
            return *info;
        }
        let info = DefInfo::read(spring, def_id);
        self.info
            .lock()
            .expect("def info cache")
            .insert(def_id, info);
        info
    }
}

impl DefInfo {
    fn read(spring: &spring::Spring<'_, spring::RulesSynced>, def_id: i32) -> Self {
        let speed = spring.unit_defs().get_unit_def_speed(def_id).ok();
        let immobile = speed.is_some_and(|speed| speed <= 0.0);
        let footprint = immobile
            .then(|| {
                Some((
                    footprint(spring, def_id, "xsize")?,
                    footprint(spring, def_id, "zsize")?,
                ))
            })
            .flatten();
        let movement = speed.filter(|speed| *speed > 0.0).map(|speed| Movement {
            speed,
            reverse_speed: param(spring, def_id, "rSpeed"),
            turn_rate: param(spring, def_id, "turnRate"),
            max_acc: param(spring, def_id, "maxAcc"),
            max_dec: param(spring, def_id, "maxDec"),
        });
        Self {
            max_weapon_range: param(spring, def_id, "maxWeaponRange"),
            movement,
            footprint,
            draw_scale: {
                let horizontal = custom_scale(spring, def_id, "hscale").unwrap_or(1.0);
                let vertical = custom_scale(spring, def_id, "vscale").unwrap_or(1.0);
                (horizontal != 1.0 || vertical != 1.0).then_some((horizontal, vertical))
            },
        }
    }
}

fn param(spring: &spring::Spring<'_, spring::RulesSynced>, def_id: i32, key: &str) -> f32 {
    use spring::ResultExt;
    spring
        .unit_defs()
        .get_unit_def_param_float(def_id, key)
        .log_err(spring, &alloc::format!("unit def {def_id} {key}"))
        .unwrap_or(0.0)
}

fn footprint(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    def_id: i32,
    key: &str,
) -> Option<f32> {
    const FOOTPRINT_TO_ELMOS: f32 = 4.0;
    spring
        .unit_defs()
        .get_unit_def_param_int(def_id, key)
        .ok()
        .map(|size| size as f32 * FOOTPRINT_TO_ELMOS)
}

fn custom_scale(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    def_id: i32,
    key: &str,
) -> Option<f32> {
    spring
        .unit_defs()
        .get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
}
