//! The unit and weapon defs the rules refer to, resolved once, and the per-def
//! values they read on hot paths, cached on first use.

use alloc::collections::BTreeMap;
use core::cell::RefCell;

use springcabal::defs::DefCache;

/// Every unit def the rules refer to by name.
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

/// Values of one unit def that the rules read per unit.
#[derive(Clone, Copy)]
pub struct DefInfo {
    /// The def's movement, for mobile units only.
    pub movement: Option<Movement>,
    /// Footprint size in elmos, for immobile units only.
    pub footprint: Option<(f32, f32)>,
}

/// A mobile unit def's unmodified movement, which the attribute effects
/// scale (Lua's `origUnitSpeed`).
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
    info: RefCell<BTreeMap<i32, DefInfo>>,
}

static DEFS: DefCache<Defs> = DefCache::new(Defs::resolve);

/// The game's defs. Resolved on first use, which is always after the engine
/// has loaded them.
pub fn defs() -> &'static Defs {
    DEFS.get()
}

impl Defs {
    fn resolve() -> Self {
        Self {
            units: UnitKind::ALL.map(|kind| spring::get_unit_def_id_by_name(kind.def_name()).ok()),
            fireball: spring::weapon_defs::get_weapon_def_id("fireball").ok(),
            info: RefCell::new(BTreeMap::new()),
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

    pub fn info(&self, def_id: i32) -> DefInfo {
        if let Some(info) = self.info.borrow().get(&def_id) {
            return *info;
        }
        let info = DefInfo::read(def_id);
        self.info.borrow_mut().insert(def_id, info);
        info
    }
}

impl DefInfo {
    fn read(def_id: i32) -> Self {
        let speed = spring::get_unit_def_speed(def_id).ok();
        let immobile = speed.is_some_and(|speed| speed <= 0.0);
        let footprint = immobile
            .then(|| Some((footprint(def_id, "xsize")?, footprint(def_id, "zsize")?)))
            .flatten();
        let movement = speed.filter(|speed| *speed > 0.0).map(|speed| Movement {
            speed,
            reverse_speed: param(def_id, "rSpeed"),
            turn_rate: param(def_id, "turnRate"),
            max_acc: param(def_id, "maxAcc"),
            max_dec: param(def_id, "maxDec"),
        });
        Self {
            movement,
            footprint,
        }
    }
}

fn param(def_id: i32, key: &str) -> f32 {
    use spring::ResultExt;
    spring::get_unit_def_param_float(def_id, key)
        .log_err(&alloc::format!("unit def {def_id} {key}"))
        .unwrap_or(0.0)
}

/// A footprint dimension in elmos.
fn footprint(def_id: i32, key: &str) -> Option<f32> {
    const FOOTPRINT_TO_ELMOS: f32 = 4.0;
    spring::get_unit_def_param_int(def_id, key)
        .ok()
        .map(|size| size as f32 * FOOTPRINT_TO_ELMOS)
}
