use super::control::PlayerCommand;
use super::spells::Spell;

pub fn parse(data: &[u8]) -> Option<PlayerCommand> {
    let message = core::str::from_utf8(data).ok()?;
    let mut fields = message.split('|');
    match fields.next()? {
        "movement" => Some(PlayerCommand::Move(position(&mut fields)?)),
        "attack" => Some(PlayerCommand::Attack {
            target: position(&mut fields)?,
            move_to: false,
        }),
        "attmove" => Some(PlayerCommand::Attack {
            target: position(&mut fields)?,
            move_to: true,
        }),
        "stop" => Some(PlayerCommand::Stop),
        "spell" => {
            let spell = Spell::from_index(number(&mut fields)? as i32)?;
            let x = number(&mut fields)?;
            let _y = number(&mut fields)?;
            let z = number(&mut fields)?;
            Some(PlayerCommand::Cast { spell, x, z })
        }
        _ => None,
    }
}

fn position<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Option<(f32, f32)> {
    let x = number(fields)?;
    let z = number(fields)?;
    Some((x, z))
}

fn number<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Option<f32> {
    fields
        .next()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite())
}
