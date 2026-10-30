use crate::state::Global;
use crate::unit_scripts;
use spring::ResultExt;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

#[derive(Default)]
pub struct UnitScriptsGadget;
impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }
    fn init(&self, ctx: &AddonContext<'_, Global, spring::RulesSynced>) {
        crate::with_cus_module(|host, spring| host.attach_existing(spring))
            .log_err_once(ctx.spring(), "attaching existing TTLD unit scripts");
        for unit in ctx.spring().units().get_all_units().unwrap_or_default() {
            if let Ok(def) = ctx.spring().units_info().get_unit_def_id(unit) {
                publish_scale(ctx.spring(), unit, def);
            }
        }
    }
    fn game_frame(&self, _ctx: &AddonContext<'_, Global, spring::RulesSynced>, frame: i32) {
        wake_tasks_last(frame);
    }
    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global, spring::RulesSynced>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        publish_scale(ctx.spring(), unit, def);
        unit_scripts::attach(ctx.spring(), unit, def);
    }
}
// Keep CUS timers behind every queued game-frame command, including commands
// enqueued by continuations. The queue supplies the Spring for this callback.
fn wake_tasks_last(frame: i32) {
    crate::with_addon_context(|ctx| {
        ctx.commands().run(move |spring| {
            let empty = crate::with_addon_context(|ctx| ctx.commands().is_empty());
            if empty {
                crate::with_cus_module(move |host, _| {
                    host.game_frame(frame);
                    Ok(())
                })
                .log_err_once(spring, "waking TTLD unit scripts");
            } else {
                wake_tasks_last(frame);
            }
        })
    });
}

// Lua's synced Unit Enlarger publishes these values even though drawing reads
// the same immutable definition scales separately. Keep its INLOS contract.
fn publish_scale(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32, def: i32) {
    if let Some((horizontal, vertical)) = crate::defs::defs(spring).info(spring, def).draw_scale {
        for (name, value) in [
            ("scale_vertical", vertical),
            ("scale_horizontal", horizontal),
        ] {
            spring
                .rules_params()
                .set_unit_rules_param_f32(unit, name, value, spring::rules_params::los::INLOS)
                .log_err_once(spring, "publishing a unit draw scale");
        }
    }
}
