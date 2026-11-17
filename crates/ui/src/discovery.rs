//! Discovery: featured public servers and search in one dialog.
use crate::{avatars::Avatars, design, dialog, icons};
use client_core::{Command, State};
use egui::{RichText, Vec2};
use model::discovery::Server;

#[derive(Default)]
pub(super) struct DiscoveryUi {
	search: String,
	requested: Option<u64>,
}

impl DiscoveryUi {
	pub fn request(&mut self, state: &State) {
		self.requested = Some(state.generation);
		self.search.clear();
	}

	/// Shows Discovery; returns the vanity invite code of a server chosen to join.
	pub fn show(
		&mut self,
		ctx: &egui::Context,
		state: &mut State,
		avatars: &mut Avatars,
		commands: &mut Vec<Command>,
	) -> Option<String> {
		if self.requested.take() == Some(state.generation)
			&& let Some(command) = state.search_discovery("")
		{
			commands.push(command);
		}
		if !state.discovery.open {
			return None;
		}
		let mut search = None;
		let mut join = None;
		let response = dialog::Dialog::new("discovery", crate::i18n::translate("discovery-title"))
			.icon(icons::Icon::Compass)
			.width(760.0)
			.show(ctx, |body| {
				body.content(|ui| {
					let edit = ui.add(
						egui::TextEdit::singleline(&mut self.search)
							.hint_text(crate::i18n::translate("discovery-search"))
							.char_limit(100)
							.desired_width(f32::INFINITY),
					);
					if edit.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
						search = Some(self.search.clone());
					}
				});
				let discovery = &state.discovery;
				let items = discovery
					.page
					.as_ref()
					.map_or(&[][..], |page| page.items.as_slice());
				if let Some(error) = discovery.error {
					body.content(|ui| {
						design::notice(ui, design::Level::Error, &error.to_string());
					});
				} else if items.is_empty() || discovery.loading() {
					body.content(|ui| {
						ui.set_min_height(200.0);
						ui.vertical_centered(|ui| {
							ui.add_space(80.0);
							if discovery.loading() {
								ui.spinner();
							} else {
								ui.label(design::semibold(
									ui,
									crate::i18n::translate("discovery-none"),
									17.0,
								));
							}
						});
					});
				} else {
					body.scroll(170.0, |ui| {
						ui.columns(2, |columns| {
							for (index, server) in items.iter().enumerate() {
								let column = &mut columns[index % 2];
								if card(column, state, server, avatars) {
									join.clone_from(&server.vanity);
								}
								column.add_space(12.0);
							}
						});
					});
				}
			});
		if let Some(query) = search
			&& let Some(command) = state.search_discovery(&query)
		{
			commands.push(command);
		}
		if response.close || join.is_some() {
			state.close_discovery();
		}
		join
	}
}

/// One server card; true when a server with a vanity invite was clicked.
fn card(ui: &mut egui::Ui, state: &State, server: &Server, avatars: &mut Avatars) -> bool {
	let colors = design::palette(ui);
	let frame = egui::Frame::new()
		.fill(colors.raised)
		.corner_radius(8)
		.inner_margin(16)
		.show(ui, |ui| {
			ui.set_width(ui.available_width());
			ui.horizontal_top(|ui| {
				let (rect, _) = ui.allocate_exact_size(Vec2::splat(48.0), egui::Sense::hover());
				let face = model::Guild {
					default_message_notifications: None,
					stickers: None,
					emojis: None,
					id: server.id,
					name: server.name.clone(),
					icon: server.icon.clone(),
					banner: None,
				};
				avatars.paint_guild(ui, &face, rect, state.demo, 14);
				ui.vertical(|ui| {
					ui.add(
						egui::Label::new(
							design::semibold(ui, &server.name, 16.0).color(colors.text_strong),
						)
						.truncate(),
					);
					if let Some(description) = &server.description {
						ui.add(
							egui::Label::new(
								RichText::new(description).size(14.0).color(colors.muted),
							)
							.wrap(),
						);
					}
					ui.add_space(4.0);
					ui.horizontal(|ui| {
						count(ui, "discovery-online", server.online, colors.positive);
						ui.add_space(8.0);
						count(ui, "discovery-members", server.members, colors.muted);
					});
				});
			});
		});
	if server.vanity.is_none() {
		return false;
	}
	let response = frame.response.interact(egui::Sense::click());
	if response.hovered() {
		ui.painter().rect_stroke(
			response.rect,
			8,
			egui::Stroke::new(1.0, colors.border),
			egui::StrokeKind::Inside,
		);
	}
	response
		.on_hover_cursor(egui::CursorIcon::PointingHand)
		.clicked()
}

fn count(ui: &mut egui::Ui, key: &str, value: u64, dot: egui::Color32) {
	let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
	ui.painter().circle_filled(rect.center(), 4.0, dot);
	let text = crate::i18n::translate_count(
		key,
		usize::try_from(value).unwrap_or(usize::MAX),
		&[("count", &grouped(value))],
	);
	ui.add(
		egui::Label::new(
			RichText::new(text)
				.size(12.0)
				.color(design::palette(ui).muted),
		)
		.extend(),
	);
}

fn grouped(value: u64) -> String {
	let digits = value.to_string();
	let mut out = String::with_capacity(digits.len() + digits.len() / 3);
	for (index, digit) in digits.chars().enumerate() {
		if index > 0 && (digits.len() - index).is_multiple_of(3) {
			out.push(',');
		}
		out.push(digit);
	}
	out
}
