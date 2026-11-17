//! Documented soundboard listings and Send Soundboard Sound, and the unofficial soundboard
//! fields of settings-proto/1.
use crate::{DiscordApi, Failure};
use discord_protocol::{
	activity_sharing::{self, MAX_SETTINGS_RESPONSE},
	soundboard as wire,
};
use model::{
	Id,
	soundboard::{Edit, Settings, Sound},
};
use wreq::Method;

impl DiscordApi {
	pub(super) async fn soundboard_settings(&self) -> Result<Settings, Failure> {
		Ok(self.read_activity_sharing().await?.soundboard)
	}

	/// Writes one soundboard setting and returns the settings Discord confirmed.
	pub(super) async fn set_soundboard(&self, edit: Edit) -> Result<Settings, Failure> {
		const UNCONFIRMED: Failure =
			Failure::ProtocolAt("Discord soundboard settings were not confirmed; try again");
		let current = self.read_activity_sharing().await?;
		let mut wanted = current.soundboard.clone();
		wanted.apply(edit);
		if wanted == current.soundboard {
			return Ok(wanted);
		}
		let patch =
			activity_sharing::encode_soundboard(&current, edit).map_err(|_| Failure::Protocol)?;
		let bytes = self
			.request_limited(
				Method::PATCH,
				"/users/@me/settings-proto/1",
				Some(serde_json::json!({"settings":patch,"required_data_version":current.version})),
				MAX_SETTINGS_RESPONSE,
			)
			.await?;
		let saved = activity_sharing::decode_response(&bytes).map_err(|_| UNCONFIRMED)?;
		let mut confirmed = saved.soundboard;
		confirmed.join_sounds.sort_by_key(|(scope, _)| *scope);
		wanted.join_sounds.sort_by_key(|(scope, _)| *scope);
		if confirmed == wanted {
			Ok(confirmed)
		} else {
			Err(UNCONFIRMED)
		}
	}

	/// Discord's default sounds, or one server's sounds.
	pub(super) async fn soundboard_sounds(&self, guild: Option<Id>) -> Result<Vec<Sound>, Failure> {
		let path = match guild {
			Some(guild) => format!("/guilds/{guild}/soundboard-sounds"),
			None => "/soundboard-default-sounds".to_owned(),
		};
		let bytes = self
			.request_limited(Method::GET, &path, None, wire::MAX_WIRE)
			.await?;
		wire::sounds(&bytes, guild)
			.map_err(|_| Failure::ProtocolAt("Soundboard sounds used an unsupported response"))
	}

	/// Plays a sound in the voice channel this account is connected to.
	pub(super) async fn send_soundboard_sound(
		&self,
		channel: Id,
		sound: Id,
		source_guild: Option<Id>,
	) -> Result<(), Failure> {
		let mut body = serde_json::json!({ "sound_id": sound.to_string() });
		if let Some(guild) = source_guild {
			body["source_guild_id"] = guild.to_string().into();
		}
		self.request_limited(
			Method::POST,
			&format!("/channels/{channel}/send-soundboard-sound"),
			Some(body),
			4 * 1024,
		)
		.await
		.map(drop)
	}
}
