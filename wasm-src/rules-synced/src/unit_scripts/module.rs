//! The Core-CUS module: the engine's `CusHost` attaches the scripts and
//! dispatches the engine's and the rules' calls to them.

use alloc::rc::Rc;
use core::cell::RefCell;
use spring::UnitId;
use spring::cus::host::{CusHost, UnitDefScript};

use spring::cus::{InitCtx, ScriptCapabilities};

use super::TtldUnitScript;
use super::kinds::{capabilities, script_kind_for_def};
use super::state::TtldState;

/// The module exported to the engine.
pub type TtldCusModule = CusHost<TtldUnitScript>;

impl UnitDefScript for TtldUnitScript {
    fn capabilities(def: i32) -> Option<ScriptCapabilities> {
        script_kind_for_def(def).map(capabilities)
    }

    fn for_def(def: i32, ctx: &mut InitCtx<'_>, _: &mut ()) -> Self {
        let kind = script_kind_for_def(def);
        let mut state = TtldState::new(ctx);
        state.kind = kind;
        Self {
            kind,
            state: Rc::new(RefCell::new(state)),
        }
    }
}

/// Gives the unit its def's script, if it has one. Deferred while the CUS
/// state is in use further up the stack: a unit created by a unit script.
pub fn attach(unit: i32, def: i32) {
    crate::with_cus_module_or_defer(move |host| host.unit_created(UnitId(unit), def));
}

/// A function the rules call on a unit's script, like Lua's
/// `env.script.Function(...)`.
#[derive(Clone, Copy, Debug)]
pub enum ScriptCall {
    StopMoving,
    /// Adrenaline's animation, for this many frames.
    Buff {
        frames: i32,
    },
    /// A stun's animation, for this many frames.
    Stun {
        frames: i32,
    },
    /// The blood mage's cast animation: level 1 to 3, and where it aims.
    CastAnimation {
        level: i32,
        tx: f32,
        tz: f32,
    },
    /// Where the blood mage aims while the player attacks, or `None` to hold
    /// its fire.
    FireTarget(Option<(f32, f32)>),
}

/// Calls a unit's script. Deferred while the CUS state is in use further up
/// the stack. A dead unit's script ignores it; a unit without a script is
/// logged, since every TTLD unit has one.
///
/// The rules are compiled into this module, so they reach the scripts
/// directly; going through `call_unit_script` would re-enter the module.
pub fn call(unit: i32, call: ScriptCall) {
    crate::with_cus_module_or_defer(move |host| {
        if host
            .with_script(UnitId(unit), |ctx, script| script.handle(ctx, call))
            .is_none()
        {
            spring::log_warning!("ttld-wasm", "{call:?} for unit {unit}: it has no script");
        }
    });
}

/// The blood mage's fire target as its script holds it; `None` when the unit
/// has no script or the module is busy.
#[cfg(feature = "headless-integration")]
pub fn fire_target(unit: i32) -> Option<Option<(f32, f32)>> {
    crate::with_cus_module(|host| {
        host.with_state(UnitId(unit), |script| script.state.borrow().fire_target)
    })
    .flatten()
}
