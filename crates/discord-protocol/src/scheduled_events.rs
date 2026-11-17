//! Documented List Scheduled Events for Guild, requested `with_user_count`.
use crate::{DecodeError, Timestamp, UserDto, permissions::List};
use model::{
	Id,
	scheduled_events::{self as m, Event, Place, Snapshot, Status},
};
use serde::Deserialize;

pub const MAX_WIRE: usize = 1024 * 1024;

#[derive(Deserialize)]
struct Metadata {
	location: Option<String>,
}
#[derive(Deserialize)]
struct WireEvent {
	id: Id,
	guild_id: Id,
	channel_id: Option<Id>,
	name: String,
	description: Option<String>,
	scheduled_start_time: Timestamp,
	scheduled_end_time: Option<Timestamp>,
	status: u8,
	entity_type: u8,
	entity_metadata: Option<Metadata>,
	creator: Option<UserDto>,
	user_count: Option<u64>,
	image: Option<String>,
}
impl WireEvent {
	/// Unknown statuses and entity types are skipped rather than guessed.
	fn checked(self, guild: Id) -> Result<Option<Event>, DecodeError> {
		if self.guild_id != guild {
			return Err(DecodeError);
		}
		let (Some(status), Some(place)) = (
			Status::from_wire(self.status),
			match (self.entity_type, self.channel_id) {
				(1, Some(channel)) => Some(Place::Stage(channel)),
				(2, Some(channel)) => Some(Place::Voice(channel)),
				(3, _) => Some(Place::External),
				_ => None,
			},
		) else {
			return Ok(None);
		};
		let location = matches!(place, Place::External)
			.then(|| self.entity_metadata.and_then(|metadata| metadata.location))
			.flatten();
		if matches!(place, Place::External) && location.is_none() {
			return Err(DecodeError);
		}
		let event = Event {
			id: self.id,
			name: self.name,
			description: self.description.filter(|text| !text.trim().is_empty()),
			start: self.scheduled_start_time.0,
			end: self.scheduled_end_time.map(|time| time.0),
			status,
			place,
			location,
			interested: self.user_count,
			creator: self.creator.map(UserDto::into_model),
			image: self.image,
		};
		if !event.valid() {
			return Err(DecodeError);
		}
		Ok(Some(event))
	}
}

pub fn snapshot(bytes: &[u8], guild: Id) -> Result<Snapshot, DecodeError> {
	if bytes.len() > MAX_WIRE || guild.0 == 0 {
		return Err(DecodeError);
	}
	let events: List<WireEvent, { m::MAX_EVENTS }> = crate::decode(bytes)?;
	let mut items = Vec::with_capacity(events.0.len());
	for event in events.0 {
		if let Some(event) = event.checked(guild)? {
			items.push(event);
		}
	}
	items.sort_by_key(|event| event.start);
	let value = Snapshot { guild, items };
	if !value.valid() {
		return Err(DecodeError);
	}
	Ok(value)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn event(id: &str, start: &str) -> serde_json::Value {
		json!({"id":id,"guild_id":"2","channel_id":"7","name":"Game night","description":"Bring snacks",
			"scheduled_start_time":start,"scheduled_end_time":null,"status":1,"entity_type":2,
			"entity_metadata":null,"user_count":4,"privacy_level":2,"creator":{"id":"9","username":"host"}})
	}

	#[test]
	fn events_decode_sorted_skip_unknown_kinds_and_reject_bad_payloads() {
		let external = json!({"id":"5","guild_id":"2","channel_id":null,"name":"Meetup",
			"scheduled_start_time":"2026-01-02T00:00:00Z","scheduled_end_time":"2026-01-02T02:00:00Z",
			"status":2,"entity_type":3,"entity_metadata":{"location":"Library"}});
		let future = json!({"id":"6","guild_id":"2","channel_id":"7","name":"Future kind",
			"scheduled_start_time":"2026-01-03T00:00:00Z","status":1,"entity_type":9});
		let body = serde_json::to_vec(&json!([
			event("4", "2026-02-01T00:00:00Z"),
			external,
			future
		]))
		.unwrap();
		let page = snapshot(&body, Id(2)).unwrap();
		assert_eq!(
			page.items.iter().map(|event| event.id).collect::<Vec<_>>(),
			[Id(5), Id(4)]
		);
		assert_eq!(page.items[0].location.as_deref(), Some("Library"));
		assert_eq!(page.items[0].status, Status::Active);
		assert_eq!(page.items[1].place, Place::Voice(Id(7)));
		assert_eq!(page.items[1].interested, Some(4));
		assert_eq!(page.upcoming(), 2);

		assert!(snapshot(&body, Id(3)).is_err());
		let no_location = json!([{"id":"5","guild_id":"2","name":"Meetup",
			"scheduled_start_time":"2026-01-02T00:00:00Z","status":1,"entity_type":3}]);
		assert!(snapshot(&serde_json::to_vec(&no_location).unwrap(), Id(2)).is_err());
		let mut ends_early = event("4", "2026-02-01T00:00:00Z");
		ends_early["scheduled_end_time"] = json!("2026-01-01T00:00:00Z");
		assert!(snapshot(&serde_json::to_vec(&json!([ends_early])).unwrap(), Id(2)).is_err());
		let duplicate = json!([
			event("4", "2026-02-01T00:00:00Z"),
			event("4", "2026-02-02T00:00:00Z")
		]);
		assert!(snapshot(&serde_json::to_vec(&duplicate).unwrap(), Id(2)).is_err());
		let too_many: Vec<_> = (1..=101)
			.map(|id| event(&id.to_string(), "2026-02-01T00:00:00Z"))
			.collect();
		assert!(snapshot(&serde_json::to_vec(&too_many).unwrap(), Id(2)).is_err());
		assert!(snapshot(&vec![b' '; MAX_WIRE + 1], Id(2)).is_err());
	}
}
