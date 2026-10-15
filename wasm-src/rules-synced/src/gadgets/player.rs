pub mod behavior;
use core::cell::Cell;

use spring::types::Float3;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::defs;
use crate::state::{GameEnd, Global};

#[derive(Default)]
pub struct PlayerGadget {
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
        behavior::resolve_pending(global, frame);
        let mut player = global.player.access_mut();
        if let Some(unit_id) = player.unit_id()
            && let Some((spell, x, z)) = player.take_cast(frame)
            && let Some(outcome) = behavior::cast(global, unit_id, spell, x, z, frame)
        {
            #[cfg(feature = "e2e")]
            spring::log_info!("ttld-wasm", "TTLD E2E cast {spell:?} on frame {frame}");
            player.begin_cast(frame, outcome.cast_frames);
        }
        if global.game_end.get() == GameEnd::Victory {
            player.drop_held();
        }
        player.update(frame);
    }

    fn handle_lua_msg(
        &self,
        ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        if ctx.global().game_end.get() == GameEnd::Victory {
            return;
        }
        if let Some(command) = behavior::parse(data) {
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
            behavior::fireball_exploded(Float3 {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            });
        }
        false
    }
}
