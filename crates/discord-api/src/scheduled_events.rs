use crate::{DiscordApi, Failure};
use discord_protocol::scheduled_events as wire;
use model::{Id, scheduled_events::Snapshot};
use wreq::Method;

impl DiscordApi {
	pub(super) async fn scheduled_events(&self, guild: Id) -> Result<Snapshot, Failure> {
		if guild.0 == 0 {
			return Err(Failure::Protocol);
		}
		let bytes = self
			.request_limited(
				Method::GET,
				&format!("/guilds/{guild}/scheduled-events?with_user_count=true"),
				None,
				wire::MAX_WIRE,
			)
			.await?;
		wire::snapshot(&bytes, guild).map_err(|_| {
			Failure::ProtocolAt("Events exceeded safe bounds or used an unsupported response")
		})
	}
}
