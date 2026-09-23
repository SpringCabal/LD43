//! The Core-CUS module: attaches scripts to units and dispatches engine calls
//! and queued rules calls to them.

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use spring::UnitId;
use spring::cus::core_module::{CoreCusCallResult, CoreCusModule};
use spring::cus::wasm::WasmCus;
use spring::cus::{CusHandle, CusInstance, CusRegistry, UnitScriptCall};

#[cfg(feature = "pose-probe")]
use crate::defs::{UnitKind, defs};
use crate::engine;

use super::TtldUnitScript;
use super::common::EnginePieces;
use super::kinds::{capabilities, script_kind_for_def};

thread_local! {
    static PENDING_UNITS: RefCell<Vec<(i32, i32)>> = const { RefCell::new(Vec::new()) };
    static PENDING_CALLS: RefCell<Vec<(i32, alloc::string::String, Vec<f32>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Queue a named unit-script call for the next Core-CUS tick.
///
/// The rules run inside this same module, so `call_unit_script` goes through
/// the engine and back into a module that is still inside a gadget callin; the
/// engine rejects that nested call (ApiError 1), so the animation never ran.
/// Calls are applied in order on the tick, like unit attach.
pub fn queue_call(unit: i32, function_name: &str, arguments: &[f32]) {
    if unit >= 0 {
        PENDING_CALLS.with(|pending| {
            pending.borrow_mut().push((
                unit,
                alloc::string::String::from(function_name),
                arguments.to_vec(),
            ));
        });
    }
}

pub fn queue_unit_created(unit: i32, def: i32) {
    if unit >= 0 {
        PENDING_UNITS.with(|pending| pending.borrow_mut().push((unit, def)));
    }
}

fn instance_id(unit: UnitId) -> u32 {
    0x5454_0000u32.wrapping_add(unit.0 as u32)
}

#[derive(Default)]
pub struct TtldCusModule {
    registry: CusRegistry<TtldUnitScript>,
    handles: BTreeMap<u32, CusHandle>,
}

impl TtldCusModule {
    fn attach_pending(&mut self) {
        let pending = PENDING_UNITS.with(|units| core::mem::take(&mut *units.borrow_mut()));
        for (unit_id, def) in pending {
            let Some(kind) = script_kind_for_def(def) else {
                continue;
            };
            let unit = UnitId(unit_id);
            let instance = instance_id(unit);
            if self.handles.contains_key(&instance) {
                continue;
            }
            // Drop units destroyed before the tick, whose id may even belong to
            // a new unit by now; retrying them would never end.
            if spring::get_unit_def_id(unit_id).map_or(true, |current| current.0 != def) {
                continue;
            }
            let Ok(map) = spring::get_unit_piece_map(unit_id) else {
                queue_unit_created(unit_id, def);
                continue;
            };
            let resolver = EnginePieces { map };
            let host = WasmCus::new(unit, instance);
            let engine = Rc::new(RefCell::new(host.engine()));
            let instance_state = CusInstance::construct(unit, &resolver, engine);
            let handle = self.registry.attach(instance_state);
            let _ = self.registry.with(handle, |instance| {
                instance.with_state(|script| script.set_kind(kind));
            });
            self.handles.insert(instance, handle);
            if WasmCus::attach(unit, instance, capabilities(kind)).is_err() {
                self.handles.remove(&instance);
                let _ = self.registry.detach(handle);
            }
        }
    }

    fn apply_pending_calls(&mut self) {
        let pending = PENDING_CALLS.with(|calls| core::mem::take(&mut *calls.borrow_mut()));
        for (unit_id, function_name, arguments) in pending {
            let Some(handle) = self.handles.get(&instance_id(UnitId(unit_id))).copied() else {
                continue;
            };
            self.sync_instance_frame(handle);
            let _ = self.registry.with(handle, |instance| {
                let ctx = instance.context();
                instance.with_state(|script| script.named_call(&ctx, &function_name, &arguments))
            });
        }
    }

    fn sync_instance_frame(&mut self, handle: CusHandle) {
        let frame = engine::frame() as u64;
        let _ = self.registry.with(handle, |instance| instance.tick(frame));
    }

    #[cfg(feature = "pose-probe")]
    fn run_pose_trigger(&mut self, level: i32) {
        let Some(target_def) = defs().id(UnitKind::BloodMage) else {
            return;
        };
        let Ok(units) = spring::get_all_units() else {
            return;
        };
        let Some(unit) = units.into_iter().find(|unit| {
            spring::get_unit_def_id(*unit)
                .map(|def| def.0 == target_def)
                .unwrap_or(false)
        }) else {
            return;
        };
        let instance = instance_id(UnitId(unit));
        let Some(handle) = self.handles.get(&instance).copied() else {
            return;
        };
        self.sync_instance_frame(handle);
        let arguments = if level == 1 {
            [level as f32, 0.0, 0.0]
        } else {
            [level as f32, 6000.0, 6000.0]
        };
        let mut handled = false;
        let _ = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| {
                handled = script.named_call(&ctx, "CastAnimation", &arguments);
            });
        });
        let _ = spring::echo(
            &alloc::format!(
                "[UnitScriptProbe] cus trigger unit={unit} level={level} handled={handled}"
            ),
            "",
        );
    }
}

impl CoreCusModule for TtldCusModule {
    fn cus_invoke(
        &mut self,
        instance_id: u32,
        call: u32,
        float_arguments: &[f32],
        integer_arguments: &[i32],
        result: &mut CoreCusCallResult<'_>,
    ) -> bool {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return false;
        };
        let Some(call) = UnitScriptCall::from_u32(call) else {
            return false;
        };
        if matches!(
            call,
            UnitScriptCall::RawCall | UnitScriptCall::StartMoving | UnitScriptCall::StopMoving
        ) {
            self.sync_instance_frame(handle);
        }
        let mut call_result = spring::cus::UnitScriptCallResult::default();
        let handled = self
            .registry
            .with(handle, |instance| {
                instance.invoke(call, float_arguments, integer_arguments, &mut call_result)
            })
            .unwrap_or(false);
        if !handled {
            return false;
        }
        result.int_value = call_result.int_value;
        result.float_value = call_result.float_value;
        result.bool_value = call_result.bool_value;
        result.complete = call_result.complete;
        result.int_count = call_result.int_values.len().min(result.int_values.len());
        result.int_values[..result.int_count]
            .copy_from_slice(&call_result.int_values[..result.int_count]);
        true
    }

    fn cus_call_named(
        &mut self,
        instance_id: u32,
        function_name: &str,
        arguments: &[f32],
        _return_values: &mut [f32],
        found: &mut bool,
    ) -> Option<usize> {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return Some(0);
        };
        self.sync_instance_frame(handle);
        let handled = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| script.named_call(&ctx, function_name, arguments))
        })?;
        if handled {
            *found = true;
            Some(0)
        } else {
            None
        }
    }

    fn cus_tick(&mut self, frame: u32) {
        self.attach_pending();
        self.apply_pending_calls();
        #[cfg(feature = "pose-probe")]
        if let Some(level) = crate::pose_probe::cast_level_at(frame as i32) {
            self.run_pose_trigger(level);
        }
        self.registry.tick(frame as u64);
    }

    fn cus_detach(&mut self, instance_id: u32) {
        if let Some(handle) = self.handles.remove(&instance_id) {
            let _ = self.registry.detach(handle);
        }
    }
}
