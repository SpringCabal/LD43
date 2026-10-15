use spring::cus::ScriptCapabilities;

use crate::defs::{UnitKind, defs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScriptKind {
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

pub(super) fn script_kind_for_def(def: i32) -> Option<ScriptKind> {
    Some(match defs().kind(def)? {
        UnitKind::BloodMage => ScriptKind::BloodMage,
        UnitKind::House => ScriptKind::House,
        UnitKind::OrkSmall => ScriptKind::OrkSmall,
        UnitKind::OrkBig => ScriptKind::OrkBig,
        UnitKind::OrkBoss => ScriptKind::OrkBoss,
        UnitKind::Coward => ScriptKind::Coward,
        UnitKind::Crossbowman | UnitKind::Builder => ScriptKind::Crossbowman,
        UnitKind::Swordsman => ScriptKind::Swordsman,
        UnitKind::Peasant => ScriptKind::Peasant,
    })
}

pub(super) fn capabilities(kind: ScriptKind) -> ScriptCapabilities {
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
