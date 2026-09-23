use alloc::format;
use core::cell::Cell;

use crate::diagnostics;
use crate::state::{GameStatus, Global};

use super::model;

/// The RmlUi context and document behind the HUD.
#[derive(Default)]
pub struct Document {
    context: Cell<u64>,
    default_layout_hidden: Cell<bool>,
    bindings_applied: Cell<bool>,
    /// The status revision last shown.
    shown_status: Cell<Option<u32>>,
}

impl Document {
    /// Loads the HUD once RmlUi is ready.
    pub fn initialize(&self, global: &Global) {
        if self.context.get() != 0 {
            return;
        }
        match spring::is_ready(0) {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                diagnostics::warning(&format!(
                    "RmlUi readiness query failed (error code {})",
                    error.code
                ));
                return;
            }
        }
        let Some(context) = create_context() else {
            return;
        };
        let Some(bindings) = model::create(context) else {
            diagnostics::warning("RmlUi data-model creation failed");
            return;
        };
        if !load_document(context) {
            return;
        }
        self.context.set(context);
        global.hud.access_mut().bindings = Some(bindings);
        diagnostics::info("To the Last Drop RmlUi HUD initialized");
    }

    pub fn hide_default_layout(&self) {
        if self.default_layout_hidden.get() {
            return;
        }
        // Each entry is one whole action line: the callout turns a non-empty
        // second argument into a *second* command, so ("resbar", "0") ran
        // `resbar` with no argument (a toggle) followed by a bogus `0`, and
        // the default HUD stayed on screen.  `hideinterface` is not an option
        // here — it blanks the RmlUi HUD along with the engine's own widgets.
        let commands = [
            "console 0",
            "resbar 0",
            "tooltip 0",
            "clock 0",
            "info 0",
            // Keep the minimap available for the WASM widget's native-backed
            // source pass; the widget slaves the regular minimap while drawing.
            "minimap minimize 0",
            "minimap simplecolors 1",
            "minimap unitexp 0.3",
            "minimap unitsize 1.2",
            "minimap fullproxy 0",
            "showhealthbars 0",
            "showrezbars 0",
            "disticon 99999",
            "endgraph 0",
        ];
        let hidden = commands
            .into_iter()
            .all(|command| spring::send_commands(command, "").is_ok());
        self.default_layout_hidden.set(hidden);
    }

    pub fn apply_bindings(&self) {
        if self.bindings_applied.replace(true) {
            return;
        }
        let _ = spring::send_commands("unbindall", "");
        for binding in ["Any+pause  pause", "f12 screenshot"] {
            let _ = spring::send_commands(&format!("bind {binding}"), "");
        }
        let _ = spring::send_commands("unbindkeyset enter chat", "");
        let _ = spring::send_commands("unbind enter", "");
        let _ = spring::send_commands("unbindaction chat", "");
    }

    /// Sizes the document to the view.
    pub fn render(&self, width: i32, height: i32) {
        let context = self.context.get();
        if context != 0 {
            let _ = spring::context_set_dimensions(context, width, height);
        }
    }

    /// Shows the latest status: health and the end-screen counters.
    pub fn show_status(&self, global: &Global) {
        let Some(bindings) = global.hud.access().bindings else {
            return;
        };
        let status: GameStatus = *global.status.access();
        if self.shown_status.replace(Some(status.revision)) == Some(status.revision) {
            return;
        }
        model::set_int(bindings.villagers_killed, status.villagers_killed);
        model::set_int(bindings.allies_drained, status.allies_drained);
        model::set_int(bindings.orks_killed, status.orks_killed);
        let max_health = status.max_health.max(1);
        let ratio = (status.health as f32 / max_health as f32).clamp(0.0, 1.0);
        let caption = if status.health <= 0 {
            alloc::string::String::from("DEAD")
        } else {
            format!("{}%", (ratio * 100.0) as i32)
        };
        model::set_int(bindings.health, status.health);
        model::set_percent(bindings.health_percent, ratio * 100.0);
        model::set_string(bindings.health_caption, &caption);
    }
}

fn create_context() -> Option<u64> {
    let context = match spring::create_rml_context("ttld-wasm") {
        Ok(context) => context,
        Err(error) => {
            diagnostics::warning(&format!(
                "RmlUi context creation failed (error code {})",
                error.code
            ));
            return None;
        }
    };
    if !context.success || context.context_handle == 0 {
        diagnostics::warning("RmlUi context creation returned no context");
        return None;
    }
    Some(context.context_handle)
}

fn load_document(context: u64) -> bool {
    let document = match spring::load_rml_document(context, "LuaUI/wasm/ttld.rml") {
        Ok(document) => document,
        Err(error) => {
            diagnostics::warning(&format!(
                "RmlUi document load failed (error code {})",
                error.code
            ));
            return false;
        }
    };
    if !document.success || document.document_handle == 0 {
        diagnostics::warning("RmlUi document load returned no document");
        return false;
    }
    match spring::show_rml_document(document.document_handle) {
        Ok(true) => true,
        Ok(false) => {
            diagnostics::warning("RmlUi document show returned false");
            false
        }
        Err(error) => {
            diagnostics::warning(&format!(
                "RmlUi document show failed (error code {})",
                error.code
            ));
            false
        }
    }
}
