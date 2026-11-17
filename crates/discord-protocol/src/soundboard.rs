//! Documented soundboard listings and the Voice Channel Effect Send gateway event.
use crate::{DecodeError, permissions::List};
use model::{
	Id,
	soundboard::{Effect, MAX_NAME_CHARS, MAX_SOUNDS, Sound},
};
use serde::Deserialize;

pub const MAX_WIRE: usize = 512 * 1024;

#[derive(Deserialize)]
struct WireSound {
	sound_id: Id,
	name: String,
	volume: f32,
	emoji_id: Option<Id>,
	emoji_name: Option<String>,
	guild_id: Option<Id>,
	#[serde(default = "available")]
	available: bool,
}
fn available() -> bool {
	true
}

fn sound(wire: WireSound) -> Option<Sound> {
	let name = wire.name.trim();
	let emoji_name = wire.emoji_name.filter(|emoji| !emoji.is_empty());
	(wire.sound_id.0 != 0
		&& !name.is_empty()
		&& name.chars().count() <= MAX_NAME_CHARS
		&& !name.chars().any(char::is_control)
		&& wire.volume.is_finite()
		&& (0.0..=1.0).contains(&wire.volume)
		&& emoji_name.as_ref().is_none_or(|emoji| emoji.len() <= 64))
	.then(|| Sound {
		id: wire.sound_id,
		name: name.to_owned(),
		volume: wire.volume,
		emoji_id: wire.emoji_id.filter(|id| id.0 != 0),
		emoji_name,
		guild: wire.guild_id.filter(|id| id.0 != 0),
		available: wire.available,
	})
}

/// `GET /soundboard-default-sounds` returns an array; the guild listing wraps it in `items`.
pub fn sounds(bytes: &[u8], guild: Option<Id>) -> Result<Vec<Sound>, DecodeError> {
	if bytes.len() > MAX_WIRE {
		return Err(DecodeError);
	}
	#[derive(Deserialize)]
	struct Items {
		items: List<WireSound, MAX_SOUNDS>,
	}
	let wire = match guild {
		None => crate::decode::<List<WireSound, MAX_SOUNDS>>(bytes)?.0,
		Some(_) => crate::decode::<Items>(bytes)?.items.0,
	};
	Ok(wire
		.into_iter()
		.filter_map(sound)
		.filter(|sound| guild.is_none_or(|guild| sound.guild == Some(guild)))
		.collect())
}

/// A played soundboard sound; emoji-only effects carry no sound and are ignored.
pub fn effect(bytes: &[u8]) -> Result<Option<Effect>, DecodeError> {
	#[derive(Deserialize)]
	struct Wire {
		channel_id: Id,
		guild_id: Option<Id>,
		user_id: Id,
		sound_id: Option<Id>,
		sound_volume: Option<f32>,
	}
	if bytes.len() > 16 * 1024 {
		return Err(DecodeError);
	}
	let wire: Wire = crate::decode(bytes)?;
	let Some(sound) = wire.sound_id.filter(|id| id.0 != 0) else {
		return Ok(None);
	};
	let volume = wire.sound_volume.unwrap_or(1.0);
	if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
		return Err(DecodeError);
	}
	Ok(Some(Effect {
		guild: wire.guild_id.filter(|id| id.0 != 0),
		channel: wire.channel_id,
		user: wire.user_id,
		sound,
		volume,
	}))
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn listings_keep_valid_sounds_of_the_requested_server_and_effects_need_a_sound() {
		let defaults = br#"[{"name":"quack","sound_id":"1","volume":1.0,"emoji_id":null,"emoji_name":"duck","available":true},
			{"name":"","sound_id":"2","volume":1.0,"emoji_id":null,"emoji_name":null,"available":true},
			{"name":"loud","sound_id":"3","volume":4.0,"emoji_id":null,"emoji_name":null,"available":true}]"#;
		let sounds = sounds(defaults, None).unwrap();
		assert_eq!(sounds.len(), 1);
		assert_eq!(sounds[0].name, "quack");
		assert_eq!(sounds[0].guild, None);

		let guild = br#"{"items":[{"name":"Yay","sound_id":"10","volume":0.5,"emoji_id":"9","emoji_name":null,"guild_id":"7","available":false},
			{"name":"Other","sound_id":"11","volume":0.5,"emoji_id":null,"emoji_name":null,"guild_id":"8","available":true}]}"#;
		let sounds = super::sounds(guild, Some(Id(7))).unwrap();
		assert_eq!(sounds.len(), 1);
		assert_eq!(
			(sounds[0].id, sounds[0].guild, sounds[0].available),
			(Id(10), Some(Id(7)), false)
		);
		assert!(super::sounds(&vec![b' '; MAX_WIRE + 1], None).is_err());

		let played = effect(br#"{"channel_id":"2","guild_id":"7","user_id":"3","sound_id":"10","sound_volume":0.5}"#)
			.unwrap()
			.unwrap();
		assert_eq!(
			(played.channel, played.sound, played.volume),
			(Id(2), Id(10), 0.5)
		);
		assert_eq!(
			effect(br#"{"channel_id":"2","user_id":"3","emoji":{"name":"x"}}"#).unwrap(),
			None
		);
		assert!(
			effect(br#"{"channel_id":"2","user_id":"3","sound_id":"10","sound_volume":9}"#)
				.is_err()
		);
	}
}
