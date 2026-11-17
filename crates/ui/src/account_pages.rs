//! Discord's read-only account pages: Nitro, Server Boost, Subscriptions, Gift Inventory, Billing
//! and Connections, filled from the listing read when the page opens.
use crate::{
	MessagingUi, design,
	i18n::{translate as tr, translate_args, translate_count},
	settings::{Page, section_heading},
};
use client_core::{Command, State};
use egui::RichText;
use model::account_data::{Data, Payment, PaymentSource, Section, Subscription, SubscriptionKind};

impl MessagingUi {
	pub(crate) fn account_data_page(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<Command>,
		page: Page,
	) {
		let section = match page {
			Page::Nitro | Page::Subscriptions => Section::Subscriptions,
			Page::Boost => Section::Boosts,
			Page::Gifts => Section::Gifts,
			Page::Billing => Section::Billing,
			_ => Section::Connections,
		};
		if let Some(failure) = state.account_data.error(section) {
			design::notice(ui, design::Level::Error, &failure.to_string());
			if design::secondary_button(ui, &tr("account-retry")).clicked() {
				commands.extend(state.read_account_data(section));
			}
			return;
		}
		if state.account_data.get(section).is_none() {
			if !state.account_data.loading(section) {
				commands.extend(state.read_account_data(section));
			}
			ui.vertical_centered(|ui| {
				ui.add_space(40.0);
				ui.spinner();
			});
			return;
		}
		let Some(data) = state.account_data.get(section) else {
			return;
		};
		let nav = &mut self.settings.sections;
		match data {
			Data::Subscriptions(items) if page == Page::Nitro => nitro(ui, items),
			Data::Subscriptions(items) => {
				section_heading(ui, nav, 0, &tr("settings-sub-your-subscriptions"));
				list(ui, items, "account-subscriptions-empty", |ui, item| {
					design::row(ui, &subscription_name(item), Some(&renewal(item)), |_| {});
				});
			}
			Data::Boosts(slots) => {
				section_heading(ui, nav, 0, &tr("settings-sub-active-boosts"));
				list(ui, slots, "account-boosts-empty", |ui, slot| {
					let server = slot.guild.and_then(|id| {
						state
							.guilds
							.iter()
							.find(|guild| guild.id == id)
							.map(|guild| guild.name.clone())
					});
					let detail = if slot.canceled {
						Some(tr("account-boost-canceled"))
					} else {
						slot.cooldown_ends.and_then(date).map(|date| {
							translate_args("account-boost-cooldown", &[("date", &date)])
						})
					};
					let title = server.unwrap_or_else(|| tr("account-boost-unused"));
					design::row(ui, &title, detail.as_deref(), |_| {});
				});
			}
			Data::Gifts(gifts) => {
				section_heading(ui, nav, 0, &tr("settings-sub-purchased-gifts"));
				list(ui, gifts, "account-gifts-empty", |ui, gift| {
					design::row(ui, &gift.name, gift.plan.as_deref(), |ui| {
						value(ui, &format!("\u{d7}{}", gift.count));
					});
				});
			}
			Data::Billing { sources, payments } => {
				section_heading(ui, nav, 0, &tr("settings-sub-payment-methods"));
				list(ui, sources, "account-sources-empty", payment_source);
				section_heading(ui, nav, 1, &tr("settings-sub-transactions"));
				list(ui, payments, "account-payments-empty", payment);
			}
			Data::Connections { .. } if crate::streamer_mode().hides_personal() => {
				section_heading(ui, nav, 0, &tr("settings-sub-connections"));
				design::notice(ui, design::Level::Info, &tr("streamer-mode-hidden"));
			}
			Data::Connections { connections, apps } => {
				section_heading(ui, nav, 0, &tr("settings-sub-connections"));
				let connections: Vec<_> = connections.iter().filter(|c| !c.revoked).collect();
				list(
					ui,
					&connections,
					"account-connections-empty",
					|ui, connection| {
						design::row(
							ui,
							&service(&connection.kind),
							Some(&connection.name),
							|_| {},
						);
					},
				);
				section_heading(ui, nav, 1, &tr("settings-sub-authorized-apps"));
				list(ui, apps, "account-apps-empty", |ui, app| {
					design::row(ui, &app.name, Some(&app.scopes.join(", ")), |_| {});
				});
			}
		}
	}
}

fn nitro(ui: &mut egui::Ui, subscriptions: &[Subscription]) {
	let active = subscriptions
		.iter()
		.find(|item| item.kind == SubscriptionKind::Nitro && !matches!(item.status, 0 | 4));
	design::card(ui, |ui| {
		let detail = active.map_or_else(|| tr("account-nitro-none"), renewal);
		design::row(ui, "Nitro", Some(&detail), |_| {});
	});
}

fn subscription_name(item: &Subscription) -> String {
	match item.kind {
		SubscriptionKind::Nitro => "Nitro".into(),
		SubscriptionKind::ServerBoost => {
			let count = item.quantity.to_string();
			translate_count(
				"account-subscription-boosts",
				item.quantity as usize,
				&[("count", &count)],
			)
		}
		SubscriptionKind::Application => tr("account-subscription-app"),
	}
}

/// Discord's subscription line: renewal or end date, or why it is not renewing.
fn renewal(item: &Subscription) -> String {
	let end = item.period_end.and_then(date).unwrap_or_default();
	match item.status {
		2 | 6 | 7 => tr("account-past-due"),
		8 | 9 => tr("account-paused"),
		_ if item.canceled || item.status == 4 => translate_args("account-ends", &[("date", &end)]),
		_ => translate_args("account-renews", &[("date", &end)]),
	}
}

fn payment_source(ui: &mut egui::Ui, source: &PaymentSource) {
	let title = match (source.kind, &source.brand, &source.last_4) {
		(1, brand, Some(digits)) => format!(
			"{} \u{2022}\u{2022}\u{2022}\u{2022} {digits}",
			brand
				.as_deref()
				.map_or_else(|| tr("account-payment-other"), capitalized)
		),
		(2, ..) => "PayPal".into(),
		_ => tr("account-payment-other"),
	};
	let detail = source
		.expires
		.map(|(month, year)| {
			translate_args(
				"account-payment-expires",
				&[("date", &format!("{month:02}/{year}"))],
			)
		})
		.or_else(|| source.email.clone());
	design::row(ui, &title, detail.as_deref(), |ui| {
		if source.invalid {
			value(ui, &tr("account-payment-invalid"));
		} else if source.default {
			value(ui, &tr("account-payment-default"));
		}
	});
}

fn payment(ui: &mut egui::Ui, payment: &Payment) {
	let when = date(payment.created).unwrap_or_default();
	let status = match payment.status {
		0 => Some("account-payment-pending"),
		2 => Some("account-payment-failed"),
		3 => Some("account-payment-reversed"),
		4 => Some("account-payment-refunded"),
		5 => Some("account-payment-canceled"),
		_ => None,
	};
	let detail = status.map_or_else(|| when.clone(), |key| format!("{when} \u{b7} {}", tr(key)));
	design::row(ui, &payment.description, Some(&detail), |ui| {
		value(ui, &amount(payment.amount, &payment.currency));
	});
}

/// Minor units to a decimal amount; yen and won have no minor unit.
fn amount(minor: i64, currency: &str) -> String {
	let code = currency.to_ascii_uppercase();
	if matches!(currency, "jpy" | "krw") {
		return format!("{minor} {code}");
	}
	let sign = if minor < 0 { "-" } else { "" };
	let minor = minor.unsigned_abs();
	format!("{sign}{}.{:02} {code}", minor / 100, minor % 100)
}

/// Discord's display names for connected services.
fn service(kind: &str) -> String {
	match kind {
		"battlenet" => "Battle.net",
		"bluesky" => "Bluesky",
		"bungie" => "Bungie.net",
		"crunchyroll" => "Crunchyroll",
		"domain" => "Domain",
		"ebay" => "eBay",
		"epicgames" => "Epic Games",
		"facebook" => "Facebook",
		"github" => "GitHub",
		"instagram" => "Instagram",
		"leagueoflegends" => "League of Legends",
		"mastodon" => "Mastodon",
		"paypal" => "PayPal",
		"playstation" => "PlayStation Network",
		"reddit" => "Reddit",
		"riotgames" => "Riot Games",
		"roblox" => "Roblox",
		"spotify" => "Spotify",
		"steam" => "Steam",
		"tiktok" => "TikTok",
		"twitch" => "Twitch",
		"twitter" => "X",
		"xbox" => "Xbox",
		"youtube" => "YouTube",
		other => return capitalized(other),
	}
	.into()
}

fn capitalized(value: &str) -> String {
	let mut chars = value.chars();
	chars.next().map_or_else(String::new, |first| {
		first.to_uppercase().chain(chars).collect()
	})
}

fn date(seconds: i64) -> Option<String> {
	crate::local_time::discord_timestamp(seconds, b'D')
}

fn value(ui: &mut egui::Ui, text: &str) {
	let muted = design::palette(ui).muted;
	ui.label(RichText::new(text).size(14.0).color(muted));
}

fn list<T>(ui: &mut egui::Ui, items: &[T], empty: &str, mut row: impl FnMut(&mut egui::Ui, &T)) {
	if items.is_empty() {
		design::hint(ui, &tr(empty));
		return;
	}
	design::card(ui, |ui| {
		for (index, item) in items.iter().enumerate() {
			if index > 0 {
				design::card_divider(ui);
			}
			row(ui, item);
		}
	});
}
