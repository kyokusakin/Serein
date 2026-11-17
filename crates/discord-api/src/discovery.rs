//! Unofficial Discovery listing and search for the signed-in account.
use crate::{DiscordApi, Failure};
use discord_protocol::discovery as wire;
use model::discovery::{PAGE, Page, valid_query};
use wreq::Method;

impl DiscordApi {
	pub(super) async fn discovery(&self, query: &str) -> Result<Page, Failure> {
		let path = if query.is_empty() {
			format!("/discoverable-guilds?limit={PAGE}&offset=0")
		} else if valid_query(query) {
			let encoded: String = query.bytes().map(|b| format!("%{b:02X}")).collect();
			format!("/discoverable-guilds/search?query={encoded}&limit={PAGE}")
		} else {
			return Err(Failure::Protocol);
		};
		let bytes = self
			.request_limited(Method::GET, &path, None, wire::MAX_WIRE)
			.await?;
		wire::page(&bytes, query).map_err(|_| {
			Failure::ProtocolAt("Discovery exceeded safe bounds or used an unsupported response")
		})
	}
}
