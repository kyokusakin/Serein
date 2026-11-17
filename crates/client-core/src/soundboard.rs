//! Soundboard: the account's soundboard settings, sound listings, sounds played in this
//! account's call, and the entrance sound sent once per joined channel.
use crate::{State, auth::Failure, voice::Phase};
use model::{
	Id,
	soundboard::{Edit, Effect, JoinSound, Settings, Sound},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Server listings kept at once, besides the default sounds.
const MAX_LISTINGS: usize = 32;
/// Played sounds waiting for the app; older ones are dropped first.
const MAX_EFFECTS: usize = 8;

#[derive(Debug)]
pub enum Command {
	Settings {
		request: u64,
	},
	Save {
		edit: Edit,
		request: u64,
	},
	/// `None` lists Discord's default sounds.
	Sounds {
		guild: Option<Id>,
	},
	Send {
		channel: Id,
		sound: Id,
		source_guild: Option<Id>,
	},
}

#[derive(Debug)]
pub enum Event {
	Settings {
		request: u64,
		result: Result<Settings, Failure>,
	},
	Sounds {
		guild: Option<Id>,
		result: Result<Vec<Sound>, Failure>,
	},
	Sent {
		result: Result<(), Failure>,
	},
	Played(Effect),
}
impl Event {
	pub(crate) fn bytes(&self) -> usize {
		match self {
			Self::Sounds {
				result: Ok(sounds), ..
			} => sounds
				.iter()
				.map(|sound| {
					std::mem::size_of::<Sound>()
						+ sound.name.capacity()
						+ sound.emoji_name.as_ref().map_or(0, String::capacity)
				})
				.sum(),
			Self::Settings {
				result: Ok(settings),
				..
			} => settings.join_sounds.len() * std::mem::size_of::<(Id, JoinSound)>(),
			_ => 0,
		}
	}
}

#[derive(Default)]
pub struct Soundboard {
	settings: Option<Settings>,
	pub error: Option<Failure>,
	pending: Option<u64>,
	sequence: u64,
	sounds: BTreeMap<Option<Id>, Vec<Sound>>,
	loading: BTreeSet<Option<Id>>,
	effects: VecDeque<Effect>,
	/// The channel whose entrance sound was already sent for the current call.
	announced: Option<(Id, u64)>,
}
impl Soundboard {
	pub fn settings(&self) -> Option<&Settings> {
		self.settings.as_ref()
	}
	pub fn sounds(&self, guild: Option<Id>) -> Option<&[Sound]> {
		self.sounds.get(&guild).map(Vec::as_slice)
	}
	pub fn loading(&self, guild: Option<Id>) -> bool {
		self.loading.contains(&guild)
	}
	/// A known sound by id, from any loaded listing.
	pub fn sound(&self, id: Id) -> Option<&Sound> {
		self.sounds.values().flatten().find(|sound| sound.id == id)
	}
}

impl State {
	/// Reads the account's soundboard settings unless a read or save is in flight.
	pub fn read_soundboard_settings(&mut self) -> Option<crate::Command> {
		let board = &mut self.soundboard;
		if board.pending.is_some() {
			return None;
		}
		board.error = None;
		board.sequence = board.sequence.wrapping_add(1);
		board.pending = Some(board.sequence);
		Some(crate::Command::Soundboard(Command::Settings {
			request: board.sequence,
		}))
	}
	/// Shows the edit at once and saves it; only the latest read or save applies.
	pub fn save_soundboard(&mut self, edit: Edit) -> Option<crate::Command> {
		let board = &mut self.soundboard;
		let settings = board.settings.get_or_insert_with(Settings::default);
		let before = settings.clone();
		settings.apply(edit);
		if *settings == before {
			return None;
		}
		board.error = None;
		board.sequence = board.sequence.wrapping_add(1);
		board.pending = Some(board.sequence);
		Some(crate::Command::Soundboard(Command::Save {
			edit,
			request: board.sequence,
		}))
	}
	/// Lists default sounds (`None`) or one server's sounds once per session.
	pub fn read_soundboard_sounds(&mut self, guild: Option<Id>) -> Option<crate::Command> {
		let board = &mut self.soundboard;
		if board.sounds.contains_key(&guild) || !board.loading.insert(guild) {
			return None;
		}
		Some(crate::Command::Soundboard(Command::Sounds { guild }))
	}
	/// The next sound someone played in this account's call.
	pub fn take_soundboard_effect(&mut self) -> Option<Effect> {
		self.soundboard.effects.pop_front()
	}
	/// Sends the entrance sound once each time this account connects to a server voice channel.
	/// Entrance sounds need Nitro; Discord refuses them otherwise.
	pub fn entrance_sound(&mut self) -> Option<crate::Command> {
		let call = self.voice.active.as_ref()?;
		let guild = call.guild?;
		if call.phase != Phase::Connected
			|| self.premium_type == 0
			|| self.soundboard.announced == Some((call.channel, call.request))
		{
			return None;
		}
		let (channel, request) = (call.channel, call.request);
		let sound = self.soundboard.settings.as_ref()?.join_sound(guild)?;
		self.soundboard.announced = Some((channel, request));
		Some(crate::Command::Soundboard(Command::Send {
			channel,
			sound: sound.sound,
			source_guild: sound.guild,
		}))
	}
	pub(crate) fn apply_soundboard(&mut self, event: Event) {
		let board = &mut self.soundboard;
		match event {
			Event::Settings { request, result } => {
				if board.pending != Some(request) {
					return;
				}
				board.pending = None;
				match result {
					Ok(settings) => board.settings = Some(settings),
					Err(failure) => board.error = Some(failure),
				}
			}
			Event::Sounds { guild, result } => {
				if !board.loading.remove(&guild) {
					return;
				}
				if board.sounds.len() > MAX_LISTINGS
					&& let Some(oldest) = board.sounds.keys().copied().find(Option::is_some)
				{
					board.sounds.remove(&oldest);
				}
				// A listing that failed reads as empty for the rest of the session.
				board.sounds.insert(guild, result.unwrap_or_default());
			}
			Event::Sent { .. } => {}
			Event::Played(effect) => {
				let in_channel = self.voice.active.as_ref().is_some_and(|call| {
					call.channel == effect.channel
						&& call.phase == Phase::Connected
						&& !call.deafened && !call.server_deafened
				});
				if in_channel {
					if board.effects.len() == MAX_EFFECTS {
						board.effects.pop_front();
					}
					board.effects.push_back(effect);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_stale_read_never_replaces_a_newer_edit() {
		let mut state = State::default();
		let Some(crate::Command::Soundboard(Command::Settings { request: read })) =
			state.read_soundboard_settings()
		else {
			panic!("the first read is sent");
		};
		let Some(crate::Command::Soundboard(Command::Save { request: save, .. })) =
			state.save_soundboard(Edit::Volume(30.0))
		else {
			panic!("an edit is saved");
		};
		state.apply_soundboard(Event::Settings {
			request: read,
			result: Ok(Settings::default()),
		});
		assert_eq!(state.soundboard.settings().unwrap().volume, 30.0);
		let mut saved = Settings::default();
		saved.apply(Edit::Volume(30.0));
		state.apply_soundboard(Event::Settings {
			request: save,
			result: Ok(saved),
		});
		assert_eq!(state.soundboard.settings().unwrap().volume, 30.0);
	}

	#[test]
	fn entrance_sound_is_sent_once_per_connection_and_prefers_the_server_choice() {
		use crate::voice::Call;
		let mut state = State {
			premium_type: 2,
			..State::default()
		};
		let mut settings = Settings::default();
		for (scope, sound) in [(Id(0), Id(1)), (Id(7), Id(2))] {
			settings.apply(Edit::JoinSound {
				scope,
				sound: Some(JoinSound { sound, guild: None }),
			});
		}
		state.soundboard.settings = Some(settings);
		state.voice.active = Some(Call {
			channel: Id(20),
			guild: Some(Id(7)),
			connected_at: None,
			channel_started_at: None,
			server_muted: false,
			server_deafened: false,
			request: 1,
			phase: Phase::Connected,
			muted: false,
			deafened: false,
			participants: Vec::new(),
			camera: false,
			watching: None,
			error: None,
			reconnecting: None,
		});
		assert!(matches!(
			state.entrance_sound(),
			Some(crate::Command::Soundboard(Command::Send {
				channel: Id(20),
				sound: Id(2),
				..
			}))
		));
		assert!(state.entrance_sound().is_none());
		state.voice.active.as_mut().unwrap().request = 2;
		state.voice.active.as_mut().unwrap().guild = Some(Id(8));
		assert!(matches!(
			state.entrance_sound(),
			Some(crate::Command::Soundboard(Command::Send {
				sound: Id(1),
				..
			}))
		));
	}
}
