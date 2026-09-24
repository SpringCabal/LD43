//! The Core-CUS module: the engine's `CusHost` attaches the scripts and
//! dispatches the engine's and the rules' calls to them.

use spring::UnitId;
use spring::cus::host::{CusHost, UnitDefScript};
use spring::cus::{InitCtx, ScriptCapabilities, UnitScript};

use super::TtldUnitScript;
use super::kinds::{capabilities, script_kind_for_def};

/// The module exported to the engine.
pub type TtldCusModule = CusHost<TtldUnitScript>;

impl UnitDefScript for TtldUnitScript {
    fn capabilities(def: i32) -> Option<ScriptCapabilities> {
        script_kind_for_def(def).map(capabilities)
    }

    fn for_def(def: i32, ctx: &mut InitCtx<'_>) -> Self {
        let mut script = Self::new(ctx);
        if let Some(kind) = script_kind_for_def(def) {
            script.set_kind(kind);
        }
        script
    }
}

/// Gives the unit its def's script, if it has one. Deferred while the CUS
/// state is in use further up the stack: a unit created by a unit script.
pub fn attach(unit: i32, def: i32) {
    crate::with_cus_module_or_defer(move |host| host.unit_created(UnitId(unit), def));
}

/// Runs a named unit-script function, like Lua's `env.Function()`. Units
/// without a script, and dead units, ignore it, as a Lua script without the
/// function would.
///
/// The rules are compiled into this module, so they reach the scripts
/// directly; going through `call_unit_script` would re-enter the module.
pub fn call(unit: i32, function_name: &'static str, arguments: &[f32]) {
    if unit < 0 {
        return;
    }
    let arguments = arguments.to_vec();
    crate::with_cus_module_or_defer(move |host| {
        host.call(UnitId(unit), function_name, &arguments, &mut []);
    });
}
