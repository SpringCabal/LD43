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
    fn capabilities(
        spring: &spring::Spring<'_, spring::RulesSynced>,
        def: i32,
    ) -> Option<ScriptCapabilities> {
        script_kind_for_def(spring, def).map(capabilities)
    }

    fn for_def(
        spring: &spring::Spring<'_, spring::RulesSynced>,
        def: i32,
        ctx: &mut InitCtx<'_>,
        _: &mut (),
    ) -> Self {
        let kind = script_kind_for_def(spring, def);
        let mut state = TtldState::new(ctx);
        state.kind = kind;
        Self {
            kind,
            state: Rc::new(RefCell::new(state)),
        }
    }
}

pub fn attach(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32, def: i32) {
    use spring::ResultExt;
    crate::with_cus_module(move |host, spring| host.unit_created(spring, UnitId(unit), def))
        .log_err_once(spring, "attaching a TTLD unit script");
}

#[derive(Clone, Copy, Debug)]
pub enum ScriptCall {
    StopMoving,
    Buff { frames: i32 },
    Stun { frames: i32 },
    CastAnimation { level: i32, tx: f32, tz: f32 },
    FireTarget(Option<(f32, f32)>),
}

pub fn call(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32, call: ScriptCall) {
    use spring::ResultExt;
    crate::with_cus_module(move |host, spring| {
        if host
            .with_script(spring, UnitId(unit), |ctx, script| script.handle(ctx, call))?
            .is_none()
        {
            spring::log_warning!(
                spring,
                "ttld-wasm",
                "{call:?} for unit {unit}: it has no script"
            );
        }
        Ok(())
    })
    .log_err_once(spring, "calling a TTLD unit script");
}

#[cfg(feature = "headless-integration")]
pub fn fire_target(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: i32,
) -> Option<Option<(f32, f32)>> {
    use spring::ResultExt;
    crate::with_cus_module(|host, _| {
        Ok(host.with_state(UnitId(unit), |script| script.state.borrow().fire_target))
    })
    .log_err_once(spring, "reading a TTLD script fire target")
    .flatten()
}
