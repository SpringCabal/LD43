use spring_addons::rules::Gadget;

use crate::player;
use crate::state::{self, Global};

#[derive(Default)]
pub struct InputGadget;

impl InputGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for InputGadget {
    fn name(&self) -> &'static str {
        "Input"
    }

    fn handle_lua_msg(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let Some(command) = player::parse(data) else {
            return;
        };
        let _ = state::with_world(|world| {
            let frame = world.frame;
            world.player.accept(command, frame);
        });
    }
}
