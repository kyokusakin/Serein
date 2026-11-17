//! A server's Events: the channel-list entry and the list it opens.
use crate::{avatars::Avatars, design, dialog, icons};
use client_core::{Command, State};
use egui::{RichText, Vec2};
use model::{
	Id,
	scheduled_events::{Event, Place, Status},
};

#[derive(Default)]
pub(super) struct EventsUi {
	requested: Option<(u64, Id)>,
}

impl EventsUi {
	pub fn request(&mut self, state: &State, guild: Id) {
		self.requested = Some((state.generation, guild));
	}

	pub fn show(
		&mut self,
		ctx: &egui::Context,
		state: &mut State,
		active: Option<Id>,
		avatars: &mut Avatars,
		commands: &mut Vec<Command>,
	) {
		if let Some((generation, guild)) = self.requested.take()
			&& generation == state.generation
			&& active == Some(guild)
			&& let Some(command) = state.open_scheduled_events(guild)
		{
			commands.push(command);
		}
		let Some(guild) = state.scheduled_events.open else {
			return;
		};
		if active != Some(guild) || state.guild(guild).is_none() {
			state.close_scheduled_events();
			return;
		}
		let response = dialog::Dialog::new(
			"scheduled-events",
			crate::i18n::translate("scheduled-events-title"),
		)
		.icon(icons::Icon::Calendar)
		.width(560.0)
		.show(ctx, |body| {
			let events = &state.scheduled_events;
			let upcoming: Vec<&Event> = events
				.snapshot
				.as_ref()
				.filter(|snapshot| snapshot.guild == guild)
				.map(|snapshot| snapshot.items.iter().filter(|e| e.upcoming()).collect())
				.unwrap_or_default();
			if let Some(error) = events.error {
				body.content(|ui| {
					design::notice(ui, design::Level::Error, &error.to_string());
				});
			} else if upcoming.is_empty() {
				body.content(|ui| {
					ui.set_min_height(160.0);
					ui.vertical_centered(|ui| {
						ui.add_space(56.0);
						if events.loading() {
							ui.spinner();
						} else {
							ui.label(design::semibold(
								ui,
								crate::i18n::translate("scheduled-events-none"),
								17.0,
							));
						}
					});
				});
			} else {
				body.scroll(120.0, |ui| {
					for event in upcoming {
						card(ui, state, event, avatars);
						ui.add_space(8.0);
					}
				});
			}
		});
		if response.close {
			state.close_scheduled_events();
		}
	}
}

/// The channel-list row: "Events", or the upcoming count once the list has been read.
pub(super) fn label(state: &State, guild: Id) -> String {
	match state.scheduled_events.upcoming(guild) {
		Some(count) if count > 0 => crate::i18n::translate_count(
			"scheduled-events-count",
			count,
			&[("count", &count.to_string())],
		),
		_ => crate::i18n::translate("scheduled-events-title"),
	}
}

fn card(ui: &mut egui::Ui, state: &State, event: &Event, avatars: &mut Avatars) {
	let colors = design::palette(ui);
	egui::Frame::new()
		.fill(colors.raised)
		.stroke(egui::Stroke::new(1.0, colors.border))
		.corner_radius(8)
		.inner_margin(16)
		.show(ui, |ui| {
			ui.set_width(ui.available_width());
			ui.horizontal(|ui| {
				if event.status == Status::Active {
					let text = crate::i18n::translate("scheduled-events-happening-now");
					ui.label(design::semibold(ui, text, 13.0).color(colors.positive));
				} else {
					icon_text(ui, icons::Icon::Calendar, when(event.start), colors.muted);
				}
				if let Some(count) = event.interested {
					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						let text = crate::i18n::translate_count(
							"scheduled-events-interested",
							usize::try_from(count).unwrap_or(usize::MAX),
							&[("count", &count.to_string())],
						);
						ui.label(RichText::new(text).size(13.0).color(colors.muted));
						let (rect, _) =
							ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
						icons::paint(ui.painter(), icons::Icon::People, rect, colors.muted);
					});
				}
			});
			ui.add_space(4.0);
			ui.horizontal(|ui| {
				if let Some(creator) = &event.creator {
					avatars.show_plain(ui, creator, 20.0, state.demo);
				}
				ui.add(
					egui::Label::new(
						design::semibold(ui, &event.name, 17.0).color(colors.text_strong),
					)
					.wrap(),
				);
			});
			if let Some(description) = &event.description {
				ui.add(
					egui::Label::new(RichText::new(description).size(14.0).color(colors.text))
						.wrap(),
				);
			}
			design::divider(ui);
			let (glyph, place) = match event.place {
				Place::Voice(channel) | Place::Stage(channel) => (
					icons::Icon::Speaker,
					state
						.channel(channel)
						.map(|channel| channel.name.clone())
						.unwrap_or_default(),
				),
				Place::External => (
					icons::Icon::Globe,
					event.location.clone().unwrap_or_default(),
				),
			};
			ui.horizontal(|ui| icon_text(ui, glyph, place, colors.muted));
		});
}

fn icon_text(ui: &mut egui::Ui, icon: icons::Icon, text: String, color: egui::Color32) {
	let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
	icons::paint(ui.painter(), icon, rect, color);
	ui.add(egui::Label::new(RichText::new(text).size(14.0).color(color)).truncate());
}

fn when(start: i128) -> String {
	time::OffsetDateTime::from_unix_timestamp_nanos(start)
		.map(crate::local_time::full)
		.unwrap_or_default()
}
