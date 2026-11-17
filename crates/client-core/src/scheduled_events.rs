//! A server's scheduled events, read once each time its Events list opens.
use crate::{Command, State, auth::Failure};
use model::{Id, scheduled_events::Snapshot};

#[derive(Default)]
pub struct Events {
	/// The server whose Events list is open.
	pub open: Option<Id>,
	/// The latest answer; it stays after closing so the channel list can count its events.
	pub snapshot: Option<Snapshot>,
	pub error: Option<Failure>,
	pending: Option<(Id, u64)>,
	sequence: u64,
}
impl Events {
	pub fn loading(&self) -> bool {
		self.pending.is_some()
	}
	/// Upcoming events in `guild`, once its list has been read this session.
	pub fn upcoming(&self, guild: Id) -> Option<usize> {
		self.snapshot
			.as_ref()
			.filter(|snapshot| snapshot.guild == guild)
			.map(Snapshot::upcoming)
	}
}

impl State {
	/// Opens `guild`'s Events list and reads its events.
	pub fn open_scheduled_events(&mut self, guild: Id) -> Option<Command> {
		if !self.guilds.iter().any(|known| known.id == guild) {
			return None;
		}
		let events = &mut self.scheduled_events;
		events.open = Some(guild);
		events.error = None;
		if events
			.snapshot
			.as_ref()
			.is_some_and(|known| known.guild != guild)
		{
			events.snapshot = None;
		}
		if events.pending.is_some_and(|(pending, _)| pending == guild) {
			return None;
		}
		events.sequence = events.sequence.wrapping_add(1);
		events.pending = Some((guild, events.sequence));
		Some(Command::ScheduledEvents {
			guild,
			request: events.sequence,
		})
	}
	pub fn close_scheduled_events(&mut self) {
		self.scheduled_events.open = None;
		self.scheduled_events.error = None;
	}
	pub(crate) fn apply_scheduled_events(
		&mut self,
		guild: Id,
		request: u64,
		result: Result<Snapshot, Failure>,
	) {
		let events = &mut self.scheduled_events;
		if events.pending != Some((guild, request)) {
			return;
		}
		events.pending = None;
		match result {
			Ok(snapshot) if snapshot.guild == guild && snapshot.valid() => {
				events.snapshot = Some(snapshot);
				events.error = None;
			}
			Ok(_) => events.error = Some(Failure::Protocol),
			Err(failure) => events.error = Some(failure),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use model::scheduled_events::{Event, Place, Status};

	fn snapshot(guild: Id) -> Snapshot {
		Snapshot {
			guild,
			items: vec![Event {
				id: Id(5),
				name: "Game night".into(),
				description: None,
				start: 0,
				end: None,
				status: Status::Scheduled,
				place: Place::Voice(Id(7)),
				location: None,
				interested: Some(1),
				creator: None,
				image: None,
			}],
		}
	}

	#[test]
	fn only_the_latest_open_applies_and_a_server_switch_drops_the_old_list() {
		let mut state = State::default();
		let guild = |id| model::Guild {
			default_message_notifications: None,
			stickers: None,
			emojis: None,
			id: Id(id),
			name: "Server".into(),
			icon: None,
			banner: None,
		};
		state.guilds = vec![guild(2), guild(3)];
		let first = Id(2);
		let second = guild(3);

		let Some(Command::ScheduledEvents { request: old, .. }) =
			state.open_scheduled_events(first)
		else {
			panic!("opening reads the events");
		};
		assert!(state.open_scheduled_events(first).is_none());
		let Some(Command::ScheduledEvents { request: new, .. }) =
			state.open_scheduled_events(second.id)
		else {
			panic!("another server reads its own events");
		};
		state.apply_scheduled_events(first, old, Ok(snapshot(first)));
		assert!(state.scheduled_events.snapshot.is_none());
		assert!(state.scheduled_events.loading());

		state.apply_scheduled_events(second.id, new, Ok(snapshot(first)));
		assert_eq!(state.scheduled_events.error, Some(Failure::Protocol));
		assert!(!state.scheduled_events.loading());

		let Some(Command::ScheduledEvents { request, .. }) = state.open_scheduled_events(second.id)
		else {
			panic!("reopening reads again");
		};
		state.apply_scheduled_events(second.id, request, Ok(snapshot(second.id)));
		state.close_scheduled_events();
		assert_eq!(state.scheduled_events.upcoming(second.id), Some(1));
		assert_eq!(state.scheduled_events.upcoming(first), None);
		state.open_scheduled_events(first);
		assert!(state.scheduled_events.snapshot.is_none());
		assert!(state.open_scheduled_events(Id(999_999)).is_none());
	}
}
