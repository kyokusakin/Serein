//! Unofficial Discovery listing and search. Entries outside model bounds are skipped.
use crate::{DecodeError, permissions::List};
use model::{
	Id,
	discovery::{self as m, Page, Server},
};
use serde::Deserialize;

/// Discoverable guild objects can carry emoji and sticker lists; only these fields are kept.
pub const MAX_WIRE: usize = 2 * 1024 * 1024;

#[derive(Deserialize)]
struct WireServer {
	id: Id,
	name: String,
	description: Option<String>,
	icon: Option<String>,
	approximate_member_count: Option<u64>,
	approximate_presence_count: Option<u64>,
	vanity_url_code: Option<String>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Wire {
	Page {
		guilds: List<WireServer, { m::PAGE }>,
	},
	Bare(List<WireServer, { m::PAGE }>),
}

pub fn page(bytes: &[u8], query: &str) -> Result<Page, DecodeError> {
	if bytes.len() > MAX_WIRE || !(query.is_empty() || m::valid_query(query)) {
		return Err(DecodeError);
	}
	let (Wire::Page { guilds } | Wire::Bare(guilds)) = crate::decode(bytes)?;
	let mut items: Vec<Server> = Vec::with_capacity(guilds.0.len());
	for wire in guilds.0 {
		let server = Server {
			id: wire.id,
			name: wire.name,
			description: wire.description.filter(|text| !text.trim().is_empty()),
			icon: wire.icon,
			members: wire.approximate_member_count.unwrap_or(0),
			online: wire.approximate_presence_count.unwrap_or(0),
			vanity: wire.vanity_url_code,
		};
		if server.valid() && !items.iter().any(|known| known.id == server.id) {
			items.push(server);
		}
	}
	let value = Page {
		query: query.to_owned(),
		items,
	};
	if !value.valid() {
		return Err(DecodeError);
	}
	Ok(value)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn server(id: &str) -> serde_json::Value {
		json!({"id":id,"name":"Synthetic hub","description":"Talk about synthetic things",
			"icon":"0123456789abcdef0123456789abcdef","approximate_member_count":1200,
			"approximate_presence_count":300,"vanity_url_code":"synthetic-hub",
			"emojis":[{"id":"1","name":"wave"}],"features":["DISCOVERABLE"]})
	}

	#[test]
	fn pages_decode_both_shapes_skip_bad_entries_and_stay_bounded() {
		let mut bad_vanity = server("3");
		bad_vanity["vanity_url_code"] = json!("../x");
		let listed = json!({"guilds":[server("1"), server("1"), bad_vanity, server("2")],
			"total":4,"offset":0,"limit":48});
		let page = page(&serde_json::to_vec(&listed).unwrap(), "").unwrap();
		assert_eq!(
			page.items.iter().map(|s| s.id).collect::<Vec<_>>(),
			[Id(1), Id(2)]
		);
		assert_eq!(page.items[0].online, 300);
		assert_eq!(page.items[0].vanity.as_deref(), Some("synthetic-hub"));

		let bare = serde_json::to_vec(&json!([server("4")])).unwrap();
		assert_eq!(super::page(&bare, "hub").unwrap().query, "hub");
		assert!(super::page(&bare, "\u{7}").is_err());
		let too_many: Vec<_> = (1..=49).map(|id| server(&id.to_string())).collect();
		assert!(
			super::page(
				&serde_json::to_vec(&json!({"guilds":too_many})).unwrap(),
				""
			)
			.is_err()
		);
		assert!(super::page(&vec![b' '; MAX_WIRE + 1], "").is_err());
	}
}
