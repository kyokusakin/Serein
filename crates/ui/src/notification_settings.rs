//! Device notification sections; message alerts, sounds and badges are stored locally.
use crate::{MessagingUi, design};
use model::notification_preferences::Sound;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Tab {
	#[default]
	Overview,
	Sounds,
	Badges,
}

impl MessagingUi {
	/// The account's idle timeout in seconds, or the default until it has been read.
	pub fn idle_timeout(&self) -> u32 {
		match self.idle_timeout_secs {
			0 => crate::DEFAULT_IDLE_TIMEOUT_SECS,
			secs => secs,
		}
	}
	/// Sound choices in effect: Streamer Mode can silence every cue without changing them.
	pub fn sound_options(&self) -> model::notification_preferences::Device {
		model::notification_preferences::Device {
			disable_sounds: self.notification_options.disable_sounds
				|| self.streamer_mode.mutes_sounds(),
			..self.notification_options
		}
	}
	/// Desktop alerts in effect: Streamer Mode can hold them without changing the preference.
	pub fn desktop_notifications(&self) -> bool {
		self.notifications_enabled && !self.streamer_mode.mutes_notifications()
	}
}

/// Whole minutes; an account value between presets rounds to the nearest minute.
fn idle_label(secs: u32) -> String {
	crate::i18n::translate_count(
		"settings-idle-after-minutes",
		((secs + 30) / 60) as usize,
		&[],
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn notification_controls_change_real_state_and_emit_preview_requests() {
		fn texts(shape: &egui::Shape, out: &mut Vec<(String, egui::Rect)>) {
			match shape {
				egui::Shape::Text(t) => {
					out.push((t.galley.job.text.clone(), t.visual_bounding_rect()))
				}
				egui::Shape::Vec(shapes) => {
					for shape in shapes {
						texts(shape, out);
					}
				}
				_ => {}
			}
		}
		for (width, dark) in [(320.0, true), (900.0, false)] {
			let ctx = egui::Context::default();
			ctx.set_visuals(if dark {
				egui::Visuals::dark()
			} else {
				egui::Visuals::light()
			});
			let mut view = MessagingUi::default();
			let render = |view: &mut MessagingUi, events| {
				let mut labels = vec![];
				let output = ctx.run_ui(
					egui::RawInput {
						screen_rect: Some(egui::Rect::from_min_size(
							egui::Pos2::ZERO,
							egui::vec2(width, 3600.0),
						)),
						events,
						focused: true,
						..Default::default()
					},
					|ui| {
						view.notification_settings(ui, true);
						assert!(
							ui.min_rect().width() <= width,
							"notification settings overflow"
						);
					},
				);
				for shape in &output.shapes {
					texts(&shape.shape, &mut labels);
				}
				output.drop_without_applying_deltas();
				labels
			};
			let labels = render(&mut view, vec![]);
			let point = labels
				.iter()
				.find(|(text, _)| text == "Preview Sound")
				.unwrap()
				.1
				.center();
			for pressed in [true, false] {
				render(
					&mut view,
					vec![
						egui::Event::PointerMoved(point),
						egui::Event::PointerButton {
							pos: point,
							button: egui::PointerButton::Primary,
							pressed,
							modifiers: egui::Modifiers::NONE,
						},
					],
				);
			}
			assert_eq!(view.notification_preview.take(), Some(Sound::Message));
		}
	}
}
impl Tab {
	pub const ALL: [Self; 3] = [Self::Overview, Self::Sounds, Self::Badges];
	pub fn label(self) -> &'static str {
		match self {
			Self::Overview => "notification-settings-tab-overview",
			Self::Sounds => "notification-settings-tab-sounds",
			Self::Badges => "notification-settings-tab-badges",
		}
	}
}
#[derive(Default)]
pub(super) struct Navigation {
	pub active: Tab,
	pub jump: Option<Tab>,
}
impl Navigation {
	fn heading(&mut self, ui: &mut egui::Ui, tab: Tab) {
		if tab != Tab::Overview {
			ui.add_space(12.0);
		}
		let heading = ui.label(design::eyebrow(ui, tab.label(), design::palette(ui).muted));
		if heading.rect.top() <= ui.clip_rect().top() + 28.0 {
			self.active = tab;
		}
		if self.jump == Some(tab) {
			ui.scroll_to_rect(heading.rect.expand(8.0), Some(egui::Align::Min));
			self.jump = None;
		}
	}
}
impl MessagingUi {
	pub(super) fn notification_settings(&mut self, ui: &mut egui::Ui, demo: bool) {
		if ui.available_width() < 500.0 {
			ui.horizontal_wrapped(|ui| {
				for tab in Tab::ALL {
					if ui
						.selectable_label(
							self.settings.notifications.active == tab,
							crate::i18n::translate_if_key(tab.label()),
						)
						.clicked()
					{
						self.settings.notifications.jump = Some(tab);
					}
				}
			});
		}
		self.settings.notifications.heading(ui, Tab::Overview);
		design::card(ui, |ui| {
			design::switch(
				ui,
				"notification-settings-notification-settings-enable-desktop-notifications",
				Some(
					"notification-settings-notification-settings-for-per-channel-or-per-server-notifications-right-click-the",
				),
				&mut self.notifications_enabled,
			);
			if !demo && !self.notification_status.is_empty() {
				design::hint(ui, self.notification_status);
			}
			design::card_divider(ui);
			design::switch(
				ui,
				"settings-auto-idle",
				Some("settings-auto-idle-description"),
				&mut self.auto_idle,
			);
			if self.auto_idle {
				design::card_divider(ui);
				let shown = self.idle_timeout();
				design::row(ui, "settings-idle-after", None, |ui| {
					crate::design::Select::from_id_salt("settings-idle-after")
						.selected_text(idle_label(shown))
						.width(ui.available_width().min(160.0))
						.show_ui(ui, |ui| {
							for minutes in [1, 2, 5, 10, 15, 30, 60] {
								ui.selectable_value(
									&mut self.idle_timeout_secs,
									minutes * 60,
									idle_label(minutes * 60),
								);
							}
						});
				});
				if !self.idle_timeout_status.is_empty() {
					design::hint(ui, self.idle_timeout_status);
				}
			}
		});
		self.settings.notifications.heading(ui, Tab::Sounds);
		design::card(ui, |ui| {
			design::slider_row(
				ui,
				"notification-settings-notification-settings-sound-volume",
				Some(
					"notification-settings-notification-settings-adjusts-the-volume-of-all-notification-sounds-and-ringtones",
				),
				&mut self.notification_options.volume,
				0..=100,
				"%",
			);
			design::card_divider(ui);
			design::switch(
				ui,
				"notification-settings-notification-settings-disable-all-notification-sounds",
				Some(
					"notification-settings-notification-settings-disables-notification-sounds-your-individual-sound-preferences-are-saved",
				),
				&mut self.notification_options.disable_sounds,
			);
			design::card_divider(ui);
			let options = &mut self.notification_options;
			crate::voice_video::sound_rows(
				ui,
				vec![
					("sound-message", &mut options.new_message, Sound::Message),
					(
						"sound-current-channel",
						&mut options.current_channel,
						Sound::CurrentChannel,
					),
					(
						"sound-incoming-ring",
						&mut options.incoming_ring,
						Sound::IncomingRing,
					),
				],
				&mut self.notification_preview,
			);
			if !self.notification_sound_status.is_empty() {
				design::card_divider(ui);
				design::notice(ui, design::Level::Warning, self.notification_sound_status);
			}
		});
		ui.add_space(4.0);
		design::card(ui, |ui| {
			if design::row(
				ui,
				"notification-settings-notification-settings-voice-video",
				Some(
					"notification-settings-notification-settings-ringtones-call-devices-and-microphone-processing",
				),
				|ui| {
					design::button(
						ui,
						&crate::i18n::translate("notification-settings-notification-settings-open"),
						design::ButtonKind::Outline,
					)
				},
			)
			.clicked()
			{
				self.open_voice_settings();
			}
		});
		self.settings.notifications.heading(ui, Tab::Badges);
		design::card(ui, |ui| {
			let supported = cfg!(any(target_os = "windows", target_os = "macos"));
			ui.add_enabled_ui(supported, |ui| {
				design::switch(
					ui,
					"notification-settings-notification-settings-enable-unread-message-badge",
					Some(if supported {
						"notification-settings-notification-settings-shows-a-red-badge-on-the-app-icon-when-you"
					} else {
						"notification-settings-notification-settings-app-icon-badges-are-not-available-on-this-platform-yet"
					}),
					&mut self.notification_options.unread_badge,
				);
			});
		});
	}
}
