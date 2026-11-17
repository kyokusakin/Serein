//! Unofficial billing, subscription, boost, gift and authorized-app listings for the signed-in
//! account, and the documented user connections list. Entries outside model bounds are skipped.
use crate::{DecodeError, Timestamp, permissions::List};
use model::{
	Id,
	account_data::{
		AuthorizedApp, BoostSlot, Connection, Gift, MAX_ITEMS, PAYMENTS, Payment, PaymentSource,
		Subscription, SubscriptionKind,
	},
};
use serde::Deserialize;

pub const MAX_WIRE: usize = 1024 * 1024;

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError> {
	if bytes.len() > MAX_WIRE {
		return Err(DecodeError);
	}
	crate::decode(bytes)
}

fn seconds(time: Option<Timestamp>) -> Option<i64> {
	time.and_then(|time| time.unix_ms()).map(|ms| ms / 1000)
}

#[derive(Deserialize)]
struct WireItem {
	quantity: Option<u32>,
}
#[derive(Deserialize)]
struct WireSubscription {
	id: Id,
	#[serde(rename = "type")]
	kind: u8,
	status: u8,
	current_period_end: Option<Timestamp>,
	canceled_at: Option<Timestamp>,
	#[serde(default)]
	items: List<WireItem, 32>,
}

pub fn subscriptions(bytes: &[u8]) -> Result<Vec<Subscription>, DecodeError> {
	let wire: List<WireSubscription, MAX_ITEMS> = decode(bytes)?;
	Ok(wire
		.0
		.into_iter()
		.filter_map(|wire| {
			let kind = match wire.kind {
				1 => SubscriptionKind::Nitro,
				2 => SubscriptionKind::ServerBoost,
				3 => SubscriptionKind::Application,
				_ => return None,
			};
			Some(Subscription {
				id: wire.id,
				kind,
				status: wire.status,
				period_end: seconds(wire.current_period_end),
				quantity: wire
					.items
					.0
					.iter()
					.map(|item| item.quantity.unwrap_or(1))
					.fold(0, u32::saturating_add),
				canceled: wire.canceled_at.is_some() || wire.status == 3,
			})
		})
		.collect())
}

#[derive(Deserialize)]
struct WireBoost {
	guild_id: Id,
	#[serde(default)]
	ended: bool,
}
#[derive(Deserialize)]
struct WireSlot {
	id: Id,
	#[serde(default)]
	canceled: bool,
	cooldown_ends_at: Option<Timestamp>,
	premium_guild_subscription: Option<WireBoost>,
}

pub fn boosts(bytes: &[u8]) -> Result<Vec<BoostSlot>, DecodeError> {
	let wire: List<WireSlot, MAX_ITEMS> = decode(bytes)?;
	Ok(wire
		.0
		.into_iter()
		.map(|slot| BoostSlot {
			id: slot.id,
			guild: slot
				.premium_guild_subscription
				.filter(|boost| !boost.ended)
				.map(|boost| boost.guild_id),
			canceled: slot.canceled,
			cooldown_ends: seconds(slot.cooldown_ends_at),
		})
		.collect())
}

#[derive(Deserialize)]
struct WireName {
	name: String,
}
#[derive(Deserialize)]
struct WireEntitlement {
	sku_id: Id,
	#[serde(default)]
	consumed: bool,
	sku: Option<WireName>,
	subscription_plan: Option<WireName>,
}

/// Unredeemed gift entitlements, one row per product and plan with how many are held.
pub fn gifts(bytes: &[u8]) -> Result<Vec<Gift>, DecodeError> {
	let wire: List<WireEntitlement, MAX_ITEMS> = decode(bytes)?;
	let mut gifts: Vec<Gift> = Vec::new();
	for entitlement in wire
		.0
		.into_iter()
		.filter(|entitlement| !entitlement.consumed)
	{
		let Some(sku) = entitlement.sku else {
			continue;
		};
		let plan = entitlement.subscription_plan.map(|plan| plan.name);
		if let Some(gift) = gifts
			.iter_mut()
			.find(|gift| gift.sku == entitlement.sku_id && gift.plan == plan)
		{
			gift.count = gift.count.saturating_add(1);
		} else {
			gifts.push(Gift {
				sku: entitlement.sku_id,
				name: sku.name,
				plan,
				count: 1,
			});
		}
	}
	Ok(gifts)
}

#[derive(Deserialize)]
struct WireSource {
	id: Id,
	#[serde(rename = "type")]
	kind: u8,
	#[serde(default)]
	invalid: bool,
	brand: Option<String>,
	last_4: Option<String>,
	expires_month: Option<u8>,
	expires_year: Option<u16>,
	email: Option<String>,
	#[serde(default)]
	default: bool,
}

pub fn payment_sources(bytes: &[u8]) -> Result<Vec<PaymentSource>, DecodeError> {
	let wire: List<WireSource, MAX_ITEMS> = decode(bytes)?;
	Ok(wire
		.0
		.into_iter()
		.map(|source| PaymentSource {
			id: source.id,
			kind: source.kind,
			brand: source.brand.filter(|brand| !brand.is_empty()),
			last_4: source.last_4.filter(|digits| !digits.is_empty()),
			expires: source.expires_month.zip(source.expires_year),
			email: source.email.filter(|email| !email.is_empty()),
			default: source.default,
			invalid: source.invalid,
		})
		.collect())
}

#[derive(Deserialize)]
struct WirePayment {
	id: Id,
	created_at: Timestamp,
	currency: String,
	amount: i64,
	status: u8,
	#[serde(default)]
	description: String,
}

pub fn payments(bytes: &[u8]) -> Result<Vec<Payment>, DecodeError> {
	let wire: List<WirePayment, PAYMENTS> = decode(bytes)?;
	Ok(wire
		.0
		.into_iter()
		.filter_map(|payment| {
			Some(Payment {
				id: payment.id,
				created: seconds(Some(payment.created_at))?,
				amount: payment.amount,
				currency: payment.currency.to_ascii_lowercase(),
				status: payment.status,
				description: payment.description,
			})
		})
		.collect())
}

#[derive(Deserialize)]
struct WireConnection {
	id: String,
	name: String,
	#[serde(rename = "type")]
	kind: String,
	#[serde(default)]
	verified: bool,
	#[serde(default)]
	revoked: bool,
}

pub fn connections(bytes: &[u8]) -> Result<Vec<Connection>, DecodeError> {
	let wire: List<WireConnection, MAX_ITEMS> = decode(bytes)?;
	Ok(wire
		.0
		.into_iter()
		.map(|connection| Connection {
			id: connection.id,
			kind: connection.kind,
			name: connection.name,
			verified: connection.verified,
			revoked: connection.revoked,
		})
		.collect())
}

#[derive(Deserialize)]
struct WireApplication {
	id: Id,
	name: String,
	icon: Option<String>,
}
#[derive(Deserialize)]
struct WireToken {
	id: Id,
	#[serde(default)]
	scopes: List<String, 32>,
	application: WireApplication,
}

pub fn authorized_apps(bytes: &[u8]) -> Result<Vec<AuthorizedApp>, DecodeError> {
	let wire: List<WireToken, MAX_ITEMS> = decode(bytes)?;
	Ok(wire
		.0
		.into_iter()
		.map(|token| AuthorizedApp {
			id: token.id,
			application: token.application.id,
			name: token.application.name,
			icon: token.application.icon,
			scopes: token.scopes.0,
		})
		.collect())
}

#[cfg(test)]
mod tests {
	use super::*;
	use model::account_data::Data;
	use serde_json::json;

	fn bytes(value: serde_json::Value) -> Vec<u8> {
		serde_json::to_vec(&value).unwrap()
	}

	#[test]
	fn listings_keep_known_kinds_group_gifts_and_reject_oversized_input() {
		let subscriptions = subscriptions(&bytes(json!([
			{"id":"1","type":1,"status":1,"current_period_end":"2026-12-01T00:00:00+00:00",
				"canceled_at":null,"items":[{"id":"9","plan_id":"5","quantity":1}],"currency":"usd"},
			{"id":"2","type":2,"status":3,"current_period_end":null,"canceled_at":null,
				"items":[{"quantity":2},{"quantity":1}]},
			{"id":"3","type":99,"status":1,"items":[]}
		])))
		.unwrap();
		assert_eq!(subscriptions.len(), 2);
		assert_eq!(subscriptions[0].period_end, Some(1_796_083_200));
		assert!(!subscriptions[0].canceled);
		assert_eq!(subscriptions[1].quantity, 3);
		assert!(subscriptions[1].canceled);

		let slots = boosts(&bytes(json!([
			{"id":"1","canceled":false,"cooldown_ends_at":null,
				"premium_guild_subscription":{"guild_id":"7","ended":false}},
			{"id":"2","canceled":false,"premium_guild_subscription":{"guild_id":"8","ended":true}},
			{"id":"3","canceled":true,"premium_guild_subscription":null}
		])))
		.unwrap();
		assert_eq!(
			slots.iter().map(|slot| slot.guild).collect::<Vec<_>>(),
			[Some(Id(7)), None, None]
		);

		let nitro = json!({"sku_id":"5","consumed":false,"sku":{"name":"Nitro"},
			"subscription_plan":{"name":"Nitro Monthly"}});
		let gifts = gifts(&bytes(json!([
			nitro,
			nitro,
			{"sku_id":"5","consumed":true,"sku":{"name":"Nitro"},
				"subscription_plan":{"name":"Nitro Monthly"}},
			{"sku_id":"5","consumed":false,"sku":{"name":"Nitro"},
				"subscription_plan":{"name":"Nitro Yearly"}},
			{"sku_id":"6","consumed":false}
		])))
		.unwrap();
		assert_eq!(
			gifts
				.iter()
				.map(|gift| (gift.plan.as_deref(), gift.count))
				.collect::<Vec<_>>(),
			[(Some("Nitro Monthly"), 2), (Some("Nitro Yearly"), 1)]
		);

		let too_many: Vec<_> = (1..=PAYMENTS + 1)
			.map(|id| {
				json!({"id":id.to_string(),"created_at":"2026-01-01T00:00:00+00:00",
					"currency":"usd","amount":999,"status":1,"description":"Nitro Monthly"})
			})
			.collect();
		assert!(payments(&bytes(json!(too_many))).is_err());
		assert!(connections(&vec![b' '; MAX_WIRE + 1]).is_err());
	}

	#[test]
	fn billing_rejects_malformed_card_digits_and_currency() {
		let source = |last_4: &str| {
			payment_sources(&bytes(json!([{"id":"1","type":1,"invalid":false,
				"brand":"visa","last_4":last_4,"expires_month":4,"expires_year":2030,
				"default":true}])))
			.unwrap()
		};
		let payment = |currency: &str| {
			payments(&bytes(
				json!([{"id":"1","created_at":"2026-01-01T00:00:00+00:00",
				"currency":currency,"amount":999,"status":1,"description":"Nitro"}]),
			))
			.unwrap()
		};
		let billing = |sources, payments| Data::Billing { sources, payments }.valid();
		assert!(billing(source("4242"), payment("USD")));
		assert!(!billing(source("42424"), payment("usd")));
		assert!(!billing(source("4242"), payment("us")));
	}
}
