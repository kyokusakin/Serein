//! Unofficial settings-proto/1 status.show_current_game interoperability.
//! Schema: discord-userdoccers/discord-protos discord_users/v1/PreloadedUserSettings.proto.
use crate::{
	DecodeError,
	guild_folders::{fields, fixed64_field, float_field, integer_wrapper, message},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use model::soundboard::{JoinSound, MAX_JOIN_SOUNDS};
use serde::Deserialize;

pub use crate::guild_folders::MAX_SETTINGS_RESPONSE;
const MAX_STATUS_BYTES: usize = 16 * 1024;

/// Seconds the account accepts for `voice_and_video.afk_timeout`; the app offers 1 to 60 minutes.
pub const IDLE_TIMEOUT_SECS: std::ops::RangeInclusive<u32> = 60..=3600;

pub struct Settings {
	pub version: u64,
	pub enabled: bool,
	/// Account inactivity timeout in seconds. Duplicates and malformed wire fail the read like
	/// the neighbouring fields; a well-formed value outside `IDLE_TIMEOUT_SECS` is ignored.
	pub afk_timeout: Option<u32>,
	/// `voice_and_video.soundboard_settings`, `disable_stream_previews` and each server's
	/// `join_sound`.
	pub soundboard: model::soundboard::Settings,
	status_wire: Vec<u8>,
	voice_wire: Vec<u8>,
	guilds_wire: Vec<u8>,
}

pub fn decode_response(bytes: &[u8]) -> Result<Settings, DecodeError> {
	#[derive(Deserialize)]
	struct Response {
		settings: String,
		#[serde(default)]
		out_of_date: bool,
	}
	if bytes.len() > MAX_SETTINGS_RESPONSE {
		return Err(DecodeError);
	}
	let response: Response = serde_json::from_slice(bytes).map_err(|_| DecodeError)?;
	if response.out_of_date {
		return Err(DecodeError);
	}
	let wire = STANDARD
		.decode(response.settings)
		.map_err(|_| DecodeError)?;
	let mut version = None;
	let mut status = None;
	let mut voice = None;
	let mut guilds = None;
	for field in fields(&wire)? {
		match field.number {
			3 => {
				if guilds.is_some() {
					return Err(DecodeError);
				}
				guilds = Some(field.message()?);
			}
			1 => {
				if version.is_some() {
					return Err(DecodeError);
				}
				let mut data_version = None;
				for field in fields(field.message()?)? {
					if field.number == 3 {
						if data_version.is_some() {
							return Err(DecodeError);
						}
						let value = field.integer()?;
						if value > u32::MAX.into() {
							return Err(DecodeError);
						}
						data_version = Some(value);
					}
				}
				version = Some(data_version.unwrap_or_default());
			}
			5 => {
				if voice.is_some() {
					return Err(DecodeError);
				}
				// Bounded by MAX_SETTINGS_RESPONSE; a cap here would fail every settings read.
				voice = Some(field.message()?);
			}
			11 => {
				if status.is_some() {
					return Err(DecodeError);
				}
				let value = field.message()?;
				if value.len() > MAX_STATUS_BYTES {
					return Err(DecodeError);
				}
				status = Some(value);
			}
			_ => {}
		}
	}
	let status = status.unwrap_or_default();
	let mut enabled = None;
	for field in fields(status)? {
		if field.number == 3 {
			if enabled.is_some() {
				return Err(DecodeError);
			}
			let mut value = None;
			for field in fields(field.message()?)? {
				if field.number == 1 {
					if value.is_some() {
						return Err(DecodeError);
					}
					value = Some(match field.integer()? {
						0 => false,
						1 => true,
						_ => return Err(DecodeError),
					});
				}
			}
			enabled = Some(value.unwrap_or(false));
		}
	}
	let voice = voice.unwrap_or_default();
	let guilds = guilds.unwrap_or_default();
	let (volume, stream_previews) = soundboard_voice(voice)?;
	Ok(Settings {
		version: version.ok_or(DecodeError)?,
		enabled: enabled.unwrap_or(true),
		afk_timeout: afk_timeout(voice)?,
		soundboard: model::soundboard::Settings {
			volume,
			stream_previews,
			join_sounds: join_sounds(guilds)?,
		},
		status_wire: status.into(),
		voice_wire: voice.into(),
		guilds_wire: guilds.into(),
	})
}

/// `soundboard_settings.volume` (0 to 100, default 100) and `disable_stream_previews`.
fn soundboard_voice(voice: &[u8]) -> Result<(f32, bool), DecodeError> {
	let mut volume = None;
	let mut previews = None;
	for field in fields(voice)? {
		match field.number {
			9 => {
				if volume.is_some() {
					return Err(DecodeError);
				}
				let mut value = 0.0;
				for field in fields(field.message()?)? {
					if field.number == 1 {
						value = field.float()?;
					}
				}
				if !value.is_finite() {
					return Err(DecodeError);
				}
				volume = Some(value.clamp(0.0, 100.0));
			}
			10 => {
				if previews.is_some() {
					return Err(DecodeError);
				}
				let mut disabled = false;
				for field in fields(field.message()?)? {
					if field.number == 1 {
						disabled = field.integer()? != 0;
					}
				}
				previews = Some(!disabled);
			}
			_ => {}
		}
	}
	Ok((volume.unwrap_or(100.0), previews.unwrap_or(true)))
}

/// Each `guilds` map entry's `join_sound`. Key 0 holds the all-servers choice.
fn join_sounds(guilds: &[u8]) -> Result<Vec<(model::Id, JoinSound)>, DecodeError> {
	let mut sounds = Vec::new();
	for entry in fields(guilds)? {
		if entry.number != 1 {
			continue;
		}
		let mut key = None;
		let mut value: &[u8] = &[];
		for field in fields(entry.message()?)? {
			match field.number {
				1 => key = Some(field.fixed64()?),
				2 => value = field.message()?,
				_ => {}
			}
		}
		let key = model::Id(key.unwrap_or_default());
		for field in fields(value)? {
			if field.number != 6 {
				continue;
			}
			let (mut sound, mut guild) = (0, 0);
			for field in fields(field.message()?)? {
				match field.number {
					1 => sound = field.fixed64()?,
					2 => guild = field.fixed64()?,
					_ => {}
				}
			}
			if sound != 0 && !sounds.iter().any(|(existing, _)| *existing == key) {
				if sounds.len() == MAX_JOIN_SOUNDS {
					return Err(DecodeError);
				}
				sounds.push((
					key,
					JoinSound {
						sound: model::Id(sound),
						guild: (guild != 0).then_some(model::Id(guild)),
					},
				));
			}
		}
	}
	Ok(sounds)
}

/// Replace one soundboard setting. Voice edits keep the other voice fields; an entrance sound
/// edit resends that server's whole entry so its other per-server settings stay.
pub fn encode_soundboard(
	current: &Settings,
	edit: model::soundboard::Edit,
) -> Result<String, DecodeError> {
	use model::soundboard::Edit;
	let voice = |replaced: u64, write: &dyn Fn(&mut Vec<u8>)| -> Result<Vec<u8>, DecodeError> {
		let mut voice = Vec::new();
		for field in fields(&current.voice_wire)? {
			if field.number != replaced {
				voice.extend_from_slice(field.raw);
			}
		}
		write(&mut voice);
		let mut patch = Vec::new();
		message(5, &voice, &mut patch);
		Ok(patch)
	};
	let mut patch = Vec::new();
	match edit {
		Edit::Volume(volume) => {
			if !volume.is_finite() || !(0.0..=100.0).contains(&volume) {
				return Err(DecodeError);
			}
			patch = voice(9, &|voice| {
				let mut settings = Vec::new();
				float_field(1, volume, &mut settings);
				message(9, &settings, voice);
			})?;
		}
		Edit::StreamPreviews(on) => {
			patch = voice(10, &|voice| integer_wrapper(10, u64::from(!on), voice))?;
		}
		Edit::JoinSound { scope, sound } => {
			let mut kept = Vec::new();
			for entry in fields(&current.guilds_wire)? {
				if entry.number != 1 {
					continue;
				}
				let mut key = 0;
				let mut value: &[u8] = &[];
				for field in fields(entry.message()?)? {
					match field.number {
						1 => key = field.fixed64()?,
						2 => value = field.message()?,
						_ => {}
					}
				}
				if key == scope.0 {
					for field in fields(value)? {
						if field.number != 6 {
							kept.extend_from_slice(field.raw);
						}
					}
				}
			}
			if let Some(sound) = sound {
				let mut call = Vec::new();
				fixed64_field(1, sound.sound.0, &mut call);
				if let Some(guild) = sound.guild {
					fixed64_field(2, guild.0, &mut call);
				}
				message(6, &call, &mut kept);
			}
			let mut entry = Vec::new();
			fixed64_field(1, scope.0, &mut entry);
			message(2, &kept, &mut entry);
			let mut guilds = Vec::new();
			message(1, &entry, &mut guilds);
			message(3, &guilds, &mut patch);
		}
	}
	Ok(STANDARD.encode(patch))
}

fn afk_timeout(voice: &[u8]) -> Result<Option<u32>, DecodeError> {
	let mut seen = false;
	let mut timeout = None;
	for field in fields(voice)? {
		if field.number == 6 {
			if seen {
				return Err(DecodeError);
			}
			seen = true;
			let mut value = None;
			for field in fields(field.message()?)? {
				if field.number == 1 {
					if value.is_some() {
						return Err(DecodeError);
					}
					value = Some(u32::try_from(field.integer()?).map_err(|_| DecodeError)?);
				}
			}
			timeout = value.filter(|value| IDLE_TIMEOUT_SECS.contains(value));
		}
	}
	Ok(timeout)
}

/// Replace only `voice_and_video.afk_timeout`; other voice settings and unknown fields stay, like
/// the status patches. The sub-message has no repeated fields, so resending it is idempotent.
pub fn encode_afk_timeout(current: &Settings, secs: u32) -> Result<String, DecodeError> {
	if !IDLE_TIMEOUT_SECS.contains(&secs) {
		return Err(DecodeError);
	}
	let mut voice = Vec::new();
	for field in fields(&current.voice_wire)? {
		if field.number != 6 {
			voice.extend_from_slice(field.raw);
		}
	}
	integer_wrapper(6, secs.into(), &mut voice);
	let mut patch = Vec::new();
	message(5, &voice, &mut patch);
	Ok(STANDARD.encode(patch))
}

/// Replace only this preference; retain status, custom status and unknown status fields.
pub fn encode_patch(current: &Settings, enabled: bool) -> Result<String, DecodeError> {
	let mut status = Vec::new();
	for field in fields(&current.status_wire)? {
		if field.number != 3 {
			status.extend_from_slice(field.raw);
		}
	}
	integer_wrapper(3, u64::from(enabled), &mut status);
	if status.len() > MAX_STATUS_BYTES {
		return Err(DecodeError);
	}
	let mut patch = Vec::new();
	message(11, &status, &mut patch);
	Ok(STANDARD.encode(patch))
}

/// Status and custom status from the same settings field activity sharing already preserves.
pub fn account_presence(settings: &Settings) -> Result<model::OwnPresence, DecodeError> {
	let mut status = None;
	let mut custom = None;
	for field in fields(&settings.status_wire)? {
		match field.number {
			1 => {
				if status.is_some() {
					return Err(DecodeError);
				}
				let wire = string_value(field.message()?)?;
				status = Some(if wire.is_empty() {
					model::PresenceStatus::Online
				} else {
					model::PresenceStatus::parse(&wire).ok_or(DecodeError)?
				});
			}
			2 => {
				if custom.is_some() {
					return Err(DecodeError);
				}
				custom = Some(custom_status(field.message()?)?);
			}
			_ => {}
		}
	}
	let (custom_status, mut expires_at_ms) = custom.unwrap_or_default();
	if custom_status.is_empty() {
		expires_at_ms = None;
	}
	let presence = model::OwnPresence {
		status: status.unwrap_or_default(),
		custom_status,
		expires_at_ms,
	};
	if !presence.custom_status.is_empty() && !presence.valid() {
		return Err(DecodeError);
	}
	Ok(presence)
}

/// Replace status and custom status. Game sharing and unknown status fields stay.
pub fn encode_account_presence(
	current: &Settings,
	presence: &model::OwnPresence,
) -> Result<String, DecodeError> {
	if !presence.valid() {
		return Err(DecodeError);
	}
	let mut status = Vec::new();
	let mut previous_custom: Option<Vec<u8>> = None;
	let mut previous_text = String::new();
	let mut previous_expires = None;
	let mut previous_status = String::new();
	let mut status_expiry: Option<Vec<u8>> = None;
	for field in fields(&current.status_wire)? {
		match field.number {
			1 => previous_status = string_value(field.message()?)?,
			4 => {
				if status_expiry.is_some() {
					return Err(DecodeError);
				}
				status_expiry = Some(field.raw.to_vec());
			}
			2 => {
				if previous_custom.is_some() {
					return Err(DecodeError);
				}
				let (text, expires) = custom_status(field.message()?)?;
				previous_text = text;
				previous_expires = expires;
				previous_custom = Some(field.raw.to_vec());
			}
			_ => status.extend_from_slice(field.raw),
		}
	}
	string_field(1, presence.status.wire(), &mut status);
	if previous_status == presence.status.wire()
		&& let Some(raw) = status_expiry
	{
		status.extend_from_slice(&raw);
	}
	let text_same = previous_text == presence.custom_status;
	let expires_same = previous_expires == presence.expires_at_ms;
	if (text_same && expires_same)
		|| (presence.custom_status.is_empty() && previous_text.is_empty())
	{
		if let Some(raw) = previous_custom {
			status.extend_from_slice(&raw);
		}
	} else if !presence.custom_status.is_empty() {
		let mut custom = Vec::new();
		if let Some(raw) = &previous_custom {
			for field in fields(message_body(raw)?)? {
				if field.number != 1 && field.number != 4 {
					custom.extend_from_slice(field.raw);
				}
			}
		}
		message(1, presence.custom_status.as_bytes(), &mut custom);
		if let Some(expires) = presence.expires_at_ms {
			fixed64_field(4, expires, &mut custom);
		}
		message(2, &custom, &mut status);
	}
	if status.len() > MAX_STATUS_BYTES {
		return Err(DecodeError);
	}
	let mut patch = Vec::new();
	message(11, &status, &mut patch);
	Ok(STANDARD.encode(patch))
}

fn string_value(bytes: &[u8]) -> Result<String, DecodeError> {
	let mut text = None;
	for field in fields(bytes)? {
		if field.number == 1 {
			if text.is_some() {
				return Err(DecodeError);
			}
			text = Some(
				std::str::from_utf8(field.message()?)
					.map_err(|_| DecodeError)?
					.to_owned(),
			);
		}
	}
	Ok(text.unwrap_or_default())
}

fn custom_status(bytes: &[u8]) -> Result<(String, Option<u64>), DecodeError> {
	let mut text = None;
	let mut expires = None;
	for field in fields(bytes)? {
		match field.number {
			1 => {
				if text.is_some() {
					return Err(DecodeError);
				}
				let value = std::str::from_utf8(field.message()?).map_err(|_| DecodeError)?;
				if value.len() > 512 {
					return Err(DecodeError);
				}
				text = Some(value.to_owned());
			}
			4 => {
				if expires.is_some() {
					return Err(DecodeError);
				}
				let value = field.fixed64()?;
				expires = Some(value).filter(|value| *value != 0);
			}
			_ => {}
		}
	}
	Ok((text.unwrap_or_default(), expires))
}

fn string_field(number: u64, text: &str, output: &mut Vec<u8>) {
	let mut wrapper = Vec::new();
	message(1, text.as_bytes(), &mut wrapper);
	message(number, &wrapper, output);
}

/// `raw` is a length-delimited field, tag included. Return its message body.
fn message_body(raw: &[u8]) -> Result<&[u8], DecodeError> {
	let field = fields(raw)?.pop().ok_or(DecodeError)?;
	field.message()
}

#[cfg(test)]
mod tests {
	use super::*;
	fn response(wire: &[u8]) -> Vec<u8> {
		serde_json::to_vec(&serde_json::json!({"settings":STANDARD.encode(wire)})).unwrap()
	}
	#[test]
	fn sharing_patch_preserves_status_unknown_fields_and_defaults() {
		let mut status = vec![10, 5, 10, 3, b'd', b'n', b'd'];
		message(2, &[10, 4, b'B', b'u', b's', b'y'], &mut status);
		message(55, b"future", &mut status);
		let preserved = status.clone();
		integer_wrapper(3, 0, &mut status);
		let mut wire = vec![10, 2, 24, 7];
		message(11, &status, &mut wire);
		let current = decode_response(&response(&wire)).unwrap();
		assert_eq!((current.version, current.enabled), (7, false));
		let parsed = account_presence(&current).unwrap();
		assert_eq!(
			(
				parsed.status,
				parsed.custom_status.as_str(),
				parsed.expires_at_ms
			),
			(model::PresenceStatus::DoNotDisturb, "Busy", None)
		);
		let edited = model::OwnPresence {
			status: model::PresenceStatus::Idle,
			custom_status: "Away".into(),
			expires_at_ms: Some(1_700_000_000_000),
		};
		let patch = STANDARD
			.decode(encode_account_presence(&current, &edited).unwrap())
			.unwrap();
		let mut roundtrip = vec![10, 2, 24, 7];
		roundtrip.extend_from_slice(&patch);
		let saved = decode_response(&response(&roundtrip)).unwrap();
		assert!(!saved.enabled);
		assert_eq!(account_presence(&saved).unwrap(), edited);
		let patch = STANDARD
			.decode(encode_patch(&current, true).unwrap())
			.unwrap();
		let root = fields(&patch).unwrap();
		assert_eq!(root.len(), 1);
		assert_eq!(root[0].number, 11);
		assert!(root[0].message().unwrap().starts_with(&preserved));
		let mut saved = vec![10, 2, 24, 8];
		saved.extend_from_slice(&patch);
		assert!(decode_response(&response(&saved)).unwrap().enabled);
		assert!(decode_response(&response(&[10, 0])).unwrap().enabled);
		assert!(
			!decode_response(&response(&[10, 0, 90, 2, 26, 0]))
				.unwrap()
				.enabled
		);
		for invalid in [
			vec![],
			vec![10, 0, 10, 0],
			vec![10, 0, 90, 4, 26, 2, 8, 2],
			vec![10, 0, 90, 3, 26, 2, 8],
		] {
			assert!(decode_response(&response(&invalid)).is_err());
		}
		assert!(decode_response(&vec![b' '; MAX_SETTINGS_RESPONSE + 1]).is_err());
		assert_eq!(current.afk_timeout, None);
		// voice_and_video with another field (1) plus afk_timeout (6) = 900 s.
		let mut voice = vec![8, 1];
		integer_wrapper(6, 900, &mut voice);
		let mut with_timeout = wire.clone();
		message(5, &voice, &mut with_timeout);
		assert_eq!(
			decode_response(&response(&with_timeout))
				.unwrap()
				.afk_timeout,
			Some(900)
		);
		// A large voice_and_video message must not fail the whole settings read.
		let mut large = Vec::new();
		message(1, &vec![0; MAX_STATUS_BYTES + 1], &mut large);
		integer_wrapper(6, 900, &mut large);
		let mut with_large = vec![10, 0];
		message(5, &large, &mut with_large);
		assert_eq!(
			decode_response(&response(&with_large)).unwrap().afk_timeout,
			Some(900)
		);
		for ignored in [0, 59, 3601] {
			let mut voice = Vec::new();
			integer_wrapper(6, ignored, &mut voice);
			let mut wire = vec![10, 0];
			message(5, &voice, &mut wire);
			assert_eq!(decode_response(&response(&wire)).unwrap().afk_timeout, None);
		}
		for invalid in [
			// duplicate voice_and_video, duplicate afk_timeout, duplicate value, over u32,
			// wrong wire type, truncated wrapper
			vec![10, 0, 42, 0, 42, 0],
			vec![10, 0, 42, 8, 50, 2, 8, 60, 50, 2, 8, 60],
			vec![10, 0, 42, 6, 50, 4, 8, 60, 8, 60],
			vec![10, 0, 42, 9, 50, 7, 8, 128, 128, 128, 128, 16, 0],
			vec![10, 0, 42, 2, 48, 60],
			vec![10, 0, 42, 4, 50, 2, 8, 128],
		] {
			assert!(decode_response(&response(&invalid)).is_err());
		}
		let current = decode_response(&response(&with_timeout)).unwrap();
		let patch = STANDARD
			.decode(encode_afk_timeout(&current, 1800).unwrap())
			.unwrap();
		let mut saved = vec![10, 2, 24, 9];
		saved.extend_from_slice(&patch);
		message(11, &preserved, &mut saved);
		let saved = decode_response(&response(&saved)).unwrap();
		assert_eq!((saved.afk_timeout, saved.enabled), (Some(1800), true));
		// The other voice_and_video field survives the patch.
		assert!(
			fields(&saved.voice_wire)
				.unwrap()
				.iter()
				.any(|field| field.number == 1)
		);
		assert_eq!(
			account_presence(&saved).unwrap().status,
			model::PresenceStatus::DoNotDisturb
		);
		assert_eq!(fields(&patch).unwrap().len(), 1);
		assert!(
			encode_afk_timeout(&current, 59).is_err()
				&& encode_afk_timeout(&current, 3601).is_err()
		);
		let mut oversized = vec![10, 0];
		message(11, &vec![0; MAX_STATUS_BYTES + 1], &mut oversized);
		assert!(decode_response(&response(&oversized)).is_err());
		assert!(decode_response(br#"{"settings":"CgA=","out_of_date":true}"#).is_err());
	}

	#[test]
	fn soundboard_settings_read_and_patch_keep_neighbouring_fields() {
		use model::soundboard::Edit;
		let mut soundboard = Vec::new();
		float_field(1, 40.0, &mut soundboard);
		let mut voice = Vec::new();
		integer_wrapper(6, 600, &mut voice);
		message(9, &soundboard, &mut voice);
		integer_wrapper(10, 1, &mut voice);
		// Server 7 has a channel setting beside its entrance sound; key 0 is all servers.
		let mut call = Vec::new();
		fixed64_field(1, 55, &mut call);
		fixed64_field(2, 7, &mut call);
		let mut server = Vec::new();
		message(1, b"channel", &mut server);
		message(6, &call, &mut server);
		let mut guilds = Vec::new();
		for (key, value) in [
			(7u64, server.clone()),
			(0, {
				let mut all = Vec::new();
				let mut call = Vec::new();
				fixed64_field(1, 3, &mut call);
				message(6, &call, &mut all);
				all
			}),
		] {
			let mut entry = Vec::new();
			fixed64_field(1, key, &mut entry);
			message(2, &value, &mut entry);
			message(1, &entry, &mut guilds);
		}
		let mut wire = vec![10, 2, 24, 4];
		message(3, &guilds, &mut wire);
		message(5, &voice, &mut wire);
		let current = decode_response(&response(&wire)).unwrap();
		let settings = &current.soundboard;
		assert_eq!((settings.volume, settings.stream_previews), (40.0, false));
		assert_eq!(
			settings.join_sound(model::Id(7)),
			Some(JoinSound {
				sound: model::Id(55),
				guild: Some(model::Id(7))
			})
		);
		assert_eq!(
			settings.join_sound(model::Id(9)).map(|sound| sound.sound),
			Some(model::Id(3))
		);
		assert_eq!(current.afk_timeout, Some(600));

		let patch = STANDARD
			.decode(encode_soundboard(&current, Edit::Volume(75.0)).unwrap())
			.unwrap();
		let mut expected_voice = Vec::new();
		integer_wrapper(6, 600, &mut expected_voice);
		integer_wrapper(10, 1, &mut expected_voice);
		let mut volume = Vec::new();
		float_field(1, 75.0, &mut volume);
		message(9, &volume, &mut expected_voice);
		let mut expected = Vec::new();
		message(5, &expected_voice, &mut expected);
		assert_eq!(patch, expected);

		// Clearing server 7's sound resends its channel setting so the entry is not emptied.
		let patch = STANDARD
			.decode(
				encode_soundboard(
					&current,
					Edit::JoinSound {
						scope: model::Id(7),
						sound: None,
					},
				)
				.unwrap(),
			)
			.unwrap();
		let mut kept = Vec::new();
		message(1, b"channel", &mut kept);
		let mut entry = Vec::new();
		fixed64_field(1, 7, &mut entry);
		message(2, &kept, &mut entry);
		let mut map = Vec::new();
		message(1, &entry, &mut map);
		let mut expected = Vec::new();
		message(3, &map, &mut expected);
		assert_eq!(patch, expected);
		assert!(encode_soundboard(&current, Edit::Volume(f32::NAN)).is_err());
	}
}
