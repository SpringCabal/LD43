use core::cell::Cell;

use spring::types::Float3;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::defs;
use crate::player::{self, Order};
use crate::state::Global;

/// The blood mage: takes the UI's commands, moves, attacks and casts.
#[derive(Default)]
pub struct PlayerGadget {
    /// The last game frame, which timestamps commands arriving between frames.
    frame: Cell<i32>,
}

impl Gadget<Global> for PlayerGadget {
    fn name(&self) -> &'static str {
        "Player"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        self.frame.set(frame);
        let global = ctx.global();
        if !global.running(frame) {
            return;
        }
        player::resolve_pending(global, frame);
        let cast = {
            let mut player = global.player.access_mut();
            player.unit_id().and_then(|unit_id| {
                let (spell, x, z) = player.take_cast(frame)?;
                Some((unit_id, spell, x, z))
            })
        };
        // Casting drains (destroys) nearby allies, whose UnitDestroyed needs
        // the player unborrowed.
        if let Some((unit_id, spell, x, z)) = cast
            && let Some(outcome) = player::cast(global, unit_id, spell, x, z, frame)
        {
            #[cfg(feature = "e2e")]
            spring::log_info!("ttld-wasm", "TTLD E2E cast {spell:?} on frame {frame}");
            global
                .player
                .access_mut()
                .begin_cast(frame, outcome.cast_frames);
        }
        let orders = {
            let mut player = global.player.access_mut();
            player.update(frame);
            player.take_orders()
        };
        orders.into_iter().for_each(Order::issue);
    }

    fn handle_lua_msg(
        &self,
        ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        if let Some(command) = player::parse(data) {
            ctx.global()
                .player
                .access_mut()
                .accept(command, self.frame.get());
        }
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        ctx.global().player.access_mut().adopt(unit, def);
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        ctx.global().player.access_mut().release(event.unit_id);
    }

    fn explosion(
        &self,
        _ctx: &AddonContext<'_, Global>,
        weapon_def_id: i32,
        pos: (f32, f32, f32),
        _owner_id: i32,
        _projectile_id: i32,
    ) -> bool {
        if defs().fireball == Some(weapon_def_id) {
            player::fireball_exploded(Float3 {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            });
        }
        // Keep the engine's own explosion graphics.
        false
    }
}
