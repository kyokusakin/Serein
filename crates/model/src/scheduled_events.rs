//! A server's scheduled events, read on demand; absent service metadata stays unknown.
use crate::{Id, User};

/// A server holds at most 100 scheduled events.
pub const MAX_EVENTS: usize = 100;
pub const MAX_BYTES: usize = 512 * 1024;
const MAX_NAME: usize = 100;
const MAX_DESCRIPTION: usize = 1000;
const MAX_LOCATION: usize = 100;
const LATEST_TIME: i128 = 253_402_300_799_999_999_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
	Scheduled,
	Active,
	Completed,
	Canceled,
}
impl Status {
	pub fn from_wire(value: u8) -> Option<Self> {
		Some(match value {
			1 => Self::Scheduled,
			2 => Self::Active,
			3 => Self::Completed,
			4 => Self::Canceled,
			_ => return None,
		})
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
	Stage(Id),
	Voice(Id),
	/// A location outside Discord, such as an address or a link.
	External,
}

#[derive(Clone)]
pub struct Event {
	pub id: Id,
	pub name: String,
	pub description: Option<String>,
	/// Nanoseconds since the Unix epoch.
	pub start: i128,
	pub end: Option<i128>,
	pub status: Status,
	pub place: Place,
	/// The external location; stage and voice events name a channel instead.
	pub location: Option<String>,
	/// Members marked interested, when the service reported it.
	pub interested: Option<u64>,
	pub creator: Option<User>,
	/// Cover image hash.
	pub image: Option<String>,
}
impl Event {
	pub fn bytes(&self) -> usize {
		size_of::<Self>()
			+ self.name.capacity()
			+ self.description.as_ref().map_or(0, String::capacity)
			+ self.location.as_ref().map_or(0, String::capacity)
			+ self.creator.as_ref().map_or(0, User::heap_bytes)
			+ self.image.as_ref().map_or(0, String::capacity)
	}
	pub fn valid(&self) -> bool {
		let channel = match self.place {
			Place::Stage(id) | Place::Voice(id) => Some(id),
			Place::External => None,
		};
		self.id.0 != 0
			&& channel.is_none_or(|id| id.0 != 0)
			&& (matches!(self.place, Place::External) == self.location.is_some())
			&& text(&self.name, MAX_NAME)
			&& !self.name.trim().is_empty()
			&& self
				.description
				.as_ref()
				.is_none_or(|value| value.chars().count() <= MAX_DESCRIPTION)
			&& self
				.location
				.as_ref()
				.is_none_or(|value| text(value, MAX_LOCATION) && !value.trim().is_empty())
			&& (0..=LATEST_TIME).contains(&self.start)
			&& self
				.end
				.is_none_or(|end| (self.start..=LATEST_TIME).contains(&end))
			&& self
				.creator
				.as_ref()
				.is_none_or(|user| user.id.0 != 0 && user.heap_bytes() <= 1024)
			&& self.image.as_ref().is_none_or(|hash| {
				!hash.is_empty()
					&& hash.len() <= 64
					&& hash
						.bytes()
						.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
			})
	}
	/// Upcoming and live events; completed and canceled ones are past.
	pub fn upcoming(&self) -> bool {
		matches!(self.status, Status::Scheduled | Status::Active)
	}
}

#[derive(Clone)]
pub struct Snapshot {
	pub guild: Id,
	/// Ordered by start time.
	pub items: Vec<Event>,
}
impl Snapshot {
	pub fn bytes(&self) -> usize {
		size_of::<Self>()
			+ self.items.iter().map(Event::bytes).sum::<usize>()
			+ self.items.capacity().saturating_sub(self.items.len()) * size_of::<Event>()
	}
	pub fn valid(&self) -> bool {
		let mut seen = std::collections::BTreeSet::new();
		self.guild.0 != 0
			&& self.items.len() <= MAX_EVENTS
			&& self.bytes() <= MAX_BYTES
			&& self
				.items
				.iter()
				.all(|event| event.valid() && seen.insert(event.id))
			&& self.items.is_sorted_by_key(|event| event.start)
	}
	pub fn upcoming(&self) -> usize {
		self.items.iter().filter(|event| event.upcoming()).count()
	}
}

fn text(value: &str, limit: usize) -> bool {
	value.chars().count() <= limit && !value.chars().any(char::is_control)
}
