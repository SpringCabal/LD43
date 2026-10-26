use alloc::format;
use core::cell::Cell;

use crate::state::{GameStatus, Global};

use super::model;

#[derive(Default)]
pub struct Document {
    context: Cell<u64>,
    shown_status: Cell<Option<u32>>,
}

impl Document {
    pub fn initialize(&self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        if self.context.get() != 0 {
            return;
        }
        match spring.rml_ui().is_ready() {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                spring::log_warning!(
                    spring,
                    "ttld-ui",
                    "RmlUi readiness query failed (error code {})",
                    error.code
                );
                return;
            }
        }
        let Some(context) = create_context(spring) else {
            return;
        };
        let Some(bindings) = model::create(spring, context, global.hud.access().intro_open) else {
            spring::log_warning!(spring, "ttld-ui", "RmlUi data-model creation failed");
            let _ = spring.rml_ui().remove_context(context);
            return;
        };
        if !load_document(spring, context) {
            let _ = spring.rml_ui().remove_context(context);
            return;
        }
        self.context.set(context);
        global.hud.access_mut().bindings = Some(bindings);
        spring::log_info!(spring, "ttld-ui", "To the Last Drop RmlUi HUD initialized");
    }

    pub fn shutdown(&self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        global.hud.access_mut().bindings = None;
        self.shown_status.set(None);
        let context = self.context.replace(0);
        if context != 0 {
            let _ = spring.rml_ui().remove_context(context);
        }
    }

    pub fn render(&self, spring: &spring::Spring<'_, spring::Ui>, width: i32, height: i32) {
        let context = self.context.get();
        if context != 0 {
            let _ = spring
                .rml_ui()
                .context_set_dimensions(context, width, height);
        }
    }

    pub fn show_status(&self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        let Some(bindings) = global.hud.access().bindings else {
            return;
        };
        let status: GameStatus = *global.status.access();
        if self.shown_status.replace(Some(status.revision)) == Some(status.revision) {
            return;
        }
        model::set_int(spring, bindings.villagers_killed, status.villagers_killed);
        model::set_int(spring, bindings.allies_drained, status.allies_drained);
        model::set_int(spring, bindings.orks_killed, status.orks_killed);
        let max_health = status.max_health.max(1);
        let ratio = (status.health as f32 / max_health as f32).clamp(0.0, 1.0);
        let caption = if status.health <= 0 {
            alloc::string::String::from("DEAD")
        } else {
            format!("{}%", (ratio * 100.0) as i32)
        };
        model::set_int(spring, bindings.health, status.health);
        model::set_percent(spring, bindings.health_percent, ratio * 100.0);
        model::set_string(spring, bindings.health_caption, &caption);
    }
}

fn create_context(spring: &spring::Spring<'_, spring::Ui>) -> Option<u64> {
    let context = match spring.rml_ui().create_context("ttld-wasm") {
        Ok(context) => context,
        Err(error) => {
            spring::log_warning!(
                spring,
                "ttld-ui",
                "RmlUi context creation failed (error code {})",
                error.code
            );
            return None;
        }
    };
    if !context.success || context.context_handle == 0 {
        spring::log_warning!(
            spring,
            "ttld-ui",
            "RmlUi context creation returned no context"
        );
        return None;
    }
    Some(context.context_handle)
}

fn load_document(spring: &spring::Spring<'_, spring::Ui>, context: u64) -> bool {
    let document = match spring
        .rml_ui()
        .context_load_document(context, "LuaUI/wasm/ttld.rml")
    {
        Ok(document) => document,
        Err(error) => {
            spring::log_warning!(
                spring,
                "ttld-ui",
                "RmlUi document load failed (error code {})",
                error.code
            );
            return false;
        }
    };
    if !document.success || document.document_handle == 0 {
        spring::log_warning!(
            spring,
            "ttld-ui",
            "RmlUi document load returned no document"
        );
        return false;
    }
    match spring
        .rml_ui()
        .document_show(document.document_handle, Default::default())
    {
        Ok(true) => true,
        Ok(false) => {
            spring::log_warning!(spring, "ttld-ui", "RmlUi document show returned false");
            false
        }
        Err(error) => {
            spring::log_warning!(
                spring,
                "ttld-ui",
                "RmlUi document show failed (error code {})",
                error.code
            );
            false
        }
    }
}
