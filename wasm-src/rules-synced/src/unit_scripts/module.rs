use alloc::rc::Rc;
use core::cell::RefCell;
use spring::UnitId;
use spring::cus::host::{CusHost, UnitDefScript};

use spring::cus::{InitCtx, ScriptCapabilities};

use super::TtldUnitScript;
use super::kinds::{capabilities, script_kind_for_def};
use super::state::TtldState;

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

pub fn attach(unit: i32, def: i32) {
    crate::with_cus_module_or_defer(move |host| host.unit_created(UnitId(unit), def));
}

#[derive(Clone, Copy, Debug)]
pub enum ScriptCall {
    StopMoving,
    Buff { frames: i32 },
    Stun { frames: i32 },
    CastAnimation { level: i32, tx: f32, tz: f32 },
    FireTarget(Option<(f32, f32)>),
}

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

#[cfg(feature = "headless-integration")]
pub fn fire_target(unit: i32) -> Option<Option<(f32, f32)>> {
    crate::with_cus_module(|host| {
        host.with_state(UnitId(unit), |script| script.state.borrow().fire_target)
    })
    .flatten()
}
