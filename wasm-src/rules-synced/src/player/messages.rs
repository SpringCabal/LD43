use springcabal::message::fields;

use super::control::PlayerCommand;
use super::spells::Spell;

/// The UI's input messages: `movement|x|z`, `attack|x|z`, `attmove|x|z`,
/// `stop` and `spell|index|x|y|z`.
pub fn parse(data: &[u8]) -> Option<PlayerCommand> {
    let message = core::str::from_utf8(data).ok()?;
    if let Some(mut fields) = fields(message, "movement") {
        return Some(PlayerCommand::Move(fields.xz()?));
    }
    for (prefix, move_to) in [("attack", false), ("attmove", true)] {
        if let Some(mut fields) = fields(message, prefix) {
            return Some(PlayerCommand::Attack {
                target: fields.xz()?,
                move_to,
            });
        }
    }
    if fields(message, "stop").is_some() {
        return Some(PlayerCommand::Stop);
    }
    let mut fields = fields(message, "spell")?;
    let spell = Spell::from_index(fields.i32()?)?;
    let (x, _y, z) = fields.xyz()?;
    Some(PlayerCommand::Cast { spell, x, z })
}
