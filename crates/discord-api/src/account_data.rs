//! Read-only account listings for the settings pages: unofficial billing, subscription, boost,
//! gift and authorized-app endpoints, and the documented user connections list.
use crate::{DiscordApi, Failure};
use discord_protocol::{DecodeError, account_data as wire};
use model::account_data::{Data, PAYMENTS, Section};
use wreq::Method;

impl DiscordApi {
	pub(super) async fn account_data(&self, section: Section) -> Result<Data, Failure> {
		let data = match section {
			Section::Subscriptions => Data::Subscriptions(
				self.read_account("/users/@me/billing/subscriptions", wire::subscriptions)
					.await?,
			),
			Section::Boosts => Data::Boosts(
				self.read_account("/users/@me/guilds/premium/subscription-slots", wire::boosts)
					.await?,
			),
			Section::Gifts => Data::Gifts(
				self.read_account("/users/@me/entitlements/gifts", wire::gifts)
					.await?,
			),
			Section::Billing => Data::Billing {
				sources: self
					.read_account("/users/@me/billing/payment-sources", wire::payment_sources)
					.await?,
				payments: self
					.read_account(
						&format!("/users/@me/billing/payments?limit={PAYMENTS}"),
						wire::payments,
					)
					.await?,
			},
			Section::Connections => Data::Connections {
				connections: self
					.read_account("/users/@me/connections", wire::connections)
					.await?,
				apps: self
					.read_account("/oauth2/tokens", wire::authorized_apps)
					.await?,
			},
		};
		if data.valid() {
			Ok(data)
		} else {
			Err(Failure::ProtocolAt(
				"Account listing exceeded safe bounds or used an unsupported response",
			))
		}
	}

	async fn read_account<T>(
		&self,
		path: &str,
		parse: fn(&[u8]) -> Result<T, DecodeError>,
	) -> Result<T, Failure> {
		let bytes = self
			.request_limited(Method::GET, path, None, wire::MAX_WIRE)
			.await?;
		parse(&bytes).map_err(|_| {
			Failure::ProtocolAt(
				"Account listing exceeded safe bounds or used an unsupported response",
			)
		})
	}
}
