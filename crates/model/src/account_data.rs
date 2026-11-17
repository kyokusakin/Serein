//! The signed-in account's subscriptions, boosts, gifts, billing and connections, read on demand
//! for the settings pages that show them. Nothing here is persisted.
use crate::Id;

/// Each list keeps at most this many entries.
pub const MAX_ITEMS: usize = 200;
/// Transactions shown on the Billing page.
pub const PAYMENTS: usize = 50;
pub const MAX_BYTES: usize = 256 * 1024;
const MAX_TEXT: usize = 200;

/// The settings page a read belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
	Subscriptions,
	Boosts,
	Gifts,
	Billing,
	Connections,
}

/// Discord's subscription types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubscriptionKind {
	Nitro,
	ServerBoost,
	Application,
}

#[derive(Clone, Debug)]
pub struct Subscription {
	pub id: Id,
	pub kind: SubscriptionKind,
	/// Discord's subscription status: 1 active, 2 past due, 3 canceled, 4 ended, 6 account hold,
	/// 7 billing retry, 8 paused, 9 pause pending; 0 unpaid.
	pub status: u8,
	/// Seconds since the Unix epoch.
	pub period_end: Option<i64>,
	/// Units across the subscription's items, such as the number of boosts.
	pub quantity: u32,
	pub canceled: bool,
}

#[derive(Clone, Debug)]
pub struct BoostSlot {
	pub id: Id,
	/// The server the boost is applied to; `None` while the slot is unused.
	pub guild: Option<Id>,
	pub canceled: bool,
	/// Seconds since the Unix epoch.
	pub cooldown_ends: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct Gift {
	pub sku: Id,
	pub name: String,
	/// The subscription plan the gift grants, such as a month of Nitro.
	pub plan: Option<String>,
	pub count: u32,
}

#[derive(Clone, Debug)]
pub struct PaymentSource {
	pub id: Id,
	/// Discord's payment source type: 1 card, 2 PayPal, others are regional methods.
	pub kind: u8,
	pub brand: Option<String>,
	pub last_4: Option<String>,
	pub expires: Option<(u8, u16)>,
	pub email: Option<String>,
	pub default: bool,
	pub invalid: bool,
}

#[derive(Clone, Debug)]
pub struct Payment {
	pub id: Id,
	/// Seconds since the Unix epoch.
	pub created: i64,
	/// Minor currency units.
	pub amount: i64,
	/// ISO 4217 code in lower case.
	pub currency: String,
	/// Discord's payment status: 0 pending, 1 completed, 2 failed, 3 reversed, 4 refunded,
	/// 5 canceled.
	pub status: u8,
	pub description: String,
}

#[derive(Clone, Debug)]
pub struct Connection {
	pub id: String,
	/// Service name such as `github` or `steam`.
	pub kind: String,
	pub name: String,
	pub verified: bool,
	pub revoked: bool,
}

#[derive(Clone, Debug)]
pub struct AuthorizedApp {
	pub id: Id,
	pub application: Id,
	pub name: String,
	/// Icon hash.
	pub icon: Option<String>,
	pub scopes: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Data {
	Subscriptions(Vec<Subscription>),
	Boosts(Vec<BoostSlot>),
	Gifts(Vec<Gift>),
	Billing {
		sources: Vec<PaymentSource>,
		payments: Vec<Payment>,
	},
	Connections {
		connections: Vec<Connection>,
		apps: Vec<AuthorizedApp>,
	},
}
impl Data {
	pub fn section(&self) -> Section {
		match self {
			Self::Subscriptions(_) => Section::Subscriptions,
			Self::Boosts(_) => Section::Boosts,
			Self::Gifts(_) => Section::Gifts,
			Self::Billing { .. } => Section::Billing,
			Self::Connections { .. } => Section::Connections,
		}
	}
	pub fn bytes(&self) -> usize {
		fn list<T>(items: &[T], each: impl Fn(&T) -> usize) -> usize {
			items.iter().map(|item| size_of::<T>() + each(item)).sum()
		}
		fn text(value: &Option<String>) -> usize {
			value.as_ref().map_or(0, String::capacity)
		}
		size_of::<Self>()
			+ match self {
				Self::Subscriptions(items) => list(items, |_| 0),
				Self::Boosts(items) => list(items, |_| 0),
				Self::Gifts(items) => list(items, |gift| gift.name.capacity() + text(&gift.plan)),
				Self::Billing { sources, payments } => {
					list(sources, |source| {
						text(&source.brand) + text(&source.last_4) + text(&source.email)
					}) + list(payments, |payment| {
						payment.currency.capacity() + payment.description.capacity()
					})
				}
				Self::Connections { connections, apps } => {
					list(connections, |connection| {
						connection.id.capacity()
							+ connection.kind.capacity()
							+ connection.name.capacity()
					}) + list(apps, |app| {
						app.name.capacity()
							+ text(&app.icon) + app.scopes.iter().map(String::capacity).sum::<usize>()
					})
				}
			}
	}
	pub fn valid(&self) -> bool {
		let bounded =
			match self {
				Self::Subscriptions(items) => items.len() <= MAX_ITEMS,
				Self::Boosts(items) => items.len() <= MAX_ITEMS,
				Self::Gifts(items) => {
					items.len() <= MAX_ITEMS
						&& items.iter().all(|gift| {
							text_ok(&gift.name) && gift.plan.as_deref().is_none_or(text_ok)
						})
				}
				Self::Billing { sources, payments } => {
					sources.len() <= MAX_ITEMS
						&& payments.len() <= PAYMENTS
						&& sources.iter().all(|source| {
							[&source.brand, &source.email]
								.into_iter()
								.all(|value| value.as_deref().is_none_or(text_ok))
								&& source.last_4.as_ref().is_none_or(|digits| {
									digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit())
								})
						}) && payments.iter().all(|payment| {
						text_ok(&payment.description)
							&& payment.currency.len() == 3
							&& payment.currency.bytes().all(|b| b.is_ascii_lowercase())
					})
				}
				Self::Connections { connections, apps } => {
					connections.len() <= MAX_ITEMS
						&& apps.len() <= MAX_ITEMS
						&& connections.iter().all(|connection| {
							text_ok(&connection.id)
								&& text_ok(&connection.name)
								&& !connection.kind.is_empty()
								&& connection.kind.bytes().all(|b| {
									b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'
								}) && connection.kind.len() <= 32
						}) && apps.iter().all(|app| {
						text_ok(&app.name)
							&& app.icon.as_deref().is_none_or(crate::valid_avatar_hash)
							&& app.scopes.len() <= 32
							&& app.scopes.iter().all(|scope| text_ok(scope))
					})
				}
			};
		bounded && self.bytes() <= MAX_BYTES
	}
}

fn text_ok(value: &str) -> bool {
	value.chars().count() <= MAX_TEXT && !value.chars().any(char::is_control)
}
