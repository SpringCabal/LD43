use ttld_shared::Command;

use super::control::PlayerCommand;
use super::spells::Spell;

pub fn parse(data: &[u8]) -> Option<PlayerCommand> {
    let message = core::str::from_utf8(data).ok()?;
    match Command::parse(message)? {
        Command::Move { x, z } => Some(PlayerCommand::Move((x, z))),
        Command::Attack { x, z, move_to } => Some(PlayerCommand::Attack {
            target: (x, z),
            move_to,
        }),
        Command::Stop => Some(PlayerCommand::Stop),
        Command::Spell { index, x, z, .. } => Some(PlayerCommand::Cast {
            spell: Spell::from_index(index)?,
            x,
            z,
        }),
        Command::Restart | Command::LeaderRenderFixture => None,
    }
}
