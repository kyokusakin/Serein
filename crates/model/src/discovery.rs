//! Public servers listed in Discovery, read on demand.
use crate::Id;

/// Discovery returns at most 48 servers per page.
pub const PAGE: usize = 48;
pub const MAX_BYTES: usize = 128 * 1024;
const MAX_NAME: usize = 100;
const MAX_DESCRIPTION: usize = 300;
const MAX_QUERY: usize = 100;

#[derive(Clone)]
pub struct Server {
	pub id: Id,
	pub name: String,
	pub description: Option<String>,
	/// Icon hash.
	pub icon: Option<String>,
	pub members: u64,
	pub online: u64,
	/// The server's vanity invite code, the only way Discovery offers to join it here.
	pub vanity: Option<String>,
}
impl Server {
	pub fn bytes(&self) -> usize {
		size_of::<Self>()
			+ self.name.capacity()
			+ self.description.as_ref().map_or(0, String::capacity)
			+ self.icon.as_ref().map_or(0, String::capacity)
			+ self.vanity.as_ref().map_or(0, String::capacity)
	}
	pub fn valid(&self) -> bool {
		self.id.0 != 0
			&& text(&self.name, MAX_NAME)
			&& !self.name.trim().is_empty()
			&& self
				.description
				.as_ref()
				.is_none_or(|value| value.chars().count() <= MAX_DESCRIPTION)
			&& self.icon.as_deref().is_none_or(crate::valid_avatar_hash)
			&& self.vanity.as_ref().is_none_or(|code| {
				(2..=32).contains(&code.len())
					&& code
						.bytes()
						.all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
			})
	}
}

#[derive(Clone)]
pub struct Page {
	/// The search that produced this page; empty for the featured list.
	pub query: String,
	pub items: Vec<Server>,
}
impl Page {
	pub fn bytes(&self) -> usize {
		size_of::<Self>()
			+ self.query.capacity()
			+ self.items.iter().map(Server::bytes).sum::<usize>()
			+ self.items.capacity().saturating_sub(self.items.len()) * size_of::<Server>()
	}
	pub fn valid(&self) -> bool {
		let mut seen = std::collections::BTreeSet::new();
		(self.query.is_empty() || valid_query(&self.query))
			&& self.items.len() <= PAGE
			&& self.bytes() <= MAX_BYTES
			&& self
				.items
				.iter()
				.all(|server| server.valid() && seen.insert(server.id))
	}
}

pub fn valid_query(query: &str) -> bool {
	!query.trim().is_empty() && text(query, MAX_QUERY)
}

fn text(value: &str, limit: usize) -> bool {
	value.chars().count() <= limit && !value.chars().any(char::is_control)
}
