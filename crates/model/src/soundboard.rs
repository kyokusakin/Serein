//! Soundboard sounds, sounds played in voice channels and the account's soundboard settings.
use crate::Id;

/// Sounds kept from one listing; Discord caps a server at a few dozen.
pub const MAX_SOUNDS: usize = 512;
/// Discord names are 2 to 32 characters.
pub const MAX_NAME_CHARS: usize = 32;
/// Entrance sound choices kept from the account settings.
pub const MAX_JOIN_SOUNDS: usize = 256;
/// One downloaded sound file; Discord accepts uploads up to 512 KiB.
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
/// Settings key of the entrance sound used in every server without its own choice.
pub const ALL_SERVERS: Id = Id(0);

#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
	pub id: Id,
	pub name: String,
	/// 0 to 1.
	pub volume: f32,
	pub emoji_id: Option<Id>,
	pub emoji_name: Option<String>,
	/// `None` for Discord's default sounds.
	pub guild: Option<Id>,
	pub available: bool,
}

/// An entrance sound: the sound and the server it comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinSound {
	pub sound: Id,
	pub guild: Option<Id>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
	/// Soundboard playback volume, 0 to 100.
	pub volume: f32,
	/// Send previews of this account's streams.
	pub stream_previews: bool,
	/// Entrance sounds by server; `ALL_SERVERS` applies where a server has none.
	pub join_sounds: Vec<(Id, JoinSound)>,
}
impl Default for Settings {
	fn default() -> Self {
		Self {
			volume: 100.0,
			stream_previews: true,
			join_sounds: Vec::new(),
		}
	}
}
impl Settings {
	/// The entrance sound for a server: its own choice, otherwise the all-servers one.
	pub fn join_sound(&self, guild: Id) -> Option<JoinSound> {
		let find = |scope: Id| {
			self.join_sounds
				.iter()
				.find(|(key, _)| *key == scope)
				.map(|(_, sound)| *sound)
		};
		find(guild).or_else(|| find(ALL_SERVERS))
	}
	pub fn apply(&mut self, edit: Edit) {
		match edit {
			Edit::Volume(volume) => self.volume = volume.clamp(0.0, 100.0),
			Edit::StreamPreviews(on) => self.stream_previews = on,
			Edit::JoinSound { scope, sound } => {
				self.join_sounds.retain(|(key, _)| *key != scope);
				if let Some(sound) = sound {
					self.join_sounds.push((scope, sound));
				}
			}
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Edit {
	Volume(f32),
	StreamPreviews(bool),
	JoinSound { scope: Id, sound: Option<JoinSound> },
}

/// Someone played a soundboard sound in a voice channel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect {
	pub guild: Option<Id>,
	pub channel: Id,
	pub user: Id,
	pub sound: Id,
	/// 0 to 1.
	pub volume: f32,
}
