//! Discord's Voice & Video page: Voice, Camera, Streaming, Sounds, Soundboard and Advanced.
use crate::{MessagingUi, design, icons::Icon, settings::section_heading};
use client_core::State;
use egui::RichText;
use model::{
	Id,
	notification_preferences::Sound,
	soundboard::{ALL_SERVERS, Edit, JoinSound},
};

impl MessagingUi {
	pub(super) fn voice_video_page(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<client_core::Command>,
	) {
		let language = self.language;
		let demo = state.demo;
		let active = state.voice.active.is_some();
		if state.soundboard.settings().is_none() && state.soundboard.error.is_none() {
			commands.extend(state.read_soundboard_settings());
		}
		let nav = &mut self.settings.sections;
		section_heading(ui, nav, 0, &language.text("settings-sub-voice"));
		self.voice_settings_content(ui, demo, active);
		let nav = &mut self.settings.sections;
		section_heading(ui, nav, 1, &language.text("settings-sub-camera"));
		design::card(ui, |ui| self.camera_settings_content(ui, demo));
		let nav = &mut self.settings.sections;
		section_heading(ui, nav, 2, &language.text("settings-sub-streaming"));
		stream_settings(ui, state, commands);
		let nav = &mut self.settings.sections;
		section_heading(ui, nav, 3, &language.text("settings-sub-sounds"));
		self.voice_sounds(ui);
		let nav = &mut self.settings.sections;
		section_heading(ui, nav, 4, &language.text("settings-sub-soundboard"));
		soundboard_settings(ui, state, commands);
		let nav = &mut self.settings.sections;
		section_heading(ui, nav, 5, &language.text("settings-sub-advanced"));
		self.voice_advanced(ui, active);
	}

	fn voice_sounds(&mut self, ui: &mut egui::Ui) {
		let options = &mut self.notification_options;
		let preview = &mut self.notification_preview;
		design::card(ui, |ui| {
			sound_rows(
				ui,
				vec![
					("sound-deafen", &mut options.deafen, Sound::Deafen),
					("sound-undeafen", &mut options.undeafen, Sound::Undeafen),
					("sound-mute", &mut options.mute, Sound::Mute),
					("sound-unmute", &mut options.unmute, Sound::Unmute),
				],
				preview,
			);
			let more = vec![
				("sound-camera-on", &mut options.camera_on, Sound::CameraOn),
				("sound-ptt-start", &mut options.ptt_start, Sound::PttStart),
				("sound-ptt-stop", &mut options.ptt_stop, Sound::PttStop),
				("sound-user-join", &mut options.user_join, Sound::UserJoin),
				(
					"sound-user-leave",
					&mut options.user_leave,
					Sound::UserLeave,
				),
				(
					"sound-outgoing-ring",
					&mut options.outgoing_ring,
					Sound::OutgoingRing,
				),
				(
					"sound-stream-started",
					&mut options.screen_share_on,
					Sound::ScreenShareOn,
				),
				(
					"sound-viewer-join",
					&mut options.viewer_join,
					Sound::ViewerJoin,
				),
			];
			design::card_divider(ui);
			let id = egui::Id::unique("voice-more-sounds");
			let open = ui.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
			let (title, detail) = if open {
				(
					crate::i18n::translate_count("voice-sounds-hide", more.len(), &[]),
					None,
				)
			} else {
				let names: Vec<String> = more
					.iter()
					.take(3)
					.map(|(key, ..)| crate::i18n::translate(key))
					.collect();
				(
					crate::i18n::translate_count("voice-sounds-show-more", more.len(), &[]),
					Some(crate::i18n::translate_args(
						"voice-sounds-show-more-description",
						&[
							("first", &names[0]),
							("second", &names[1]),
							("third", &names[2]),
						],
					)),
				)
			};
			let icon = if open {
				Icon::ChevronDown
			} else {
				Icon::ChevronRight
			};
			if design::row(ui, &title, detail.as_deref(), |ui| {
				crate::icons::button(ui, icon, 32.0, &title)
			})
			.clicked()
			{
				ui.data_mut(|data| data.insert_temp(id, !open));
			}
			if open {
				design::card_divider(ui);
				sound_rows(ui, more, preview);
			}
		});
		ui.label(
			design::semibold(ui, crate::i18n::translate("voice-related-settings"), 14.0)
				.color(design::palette(ui).text_strong),
		);
		if related_link(
			ui,
			Icon::Bell,
			"page-notifications",
			"voice-related-notifications-description",
		)
		.clicked()
		{
			self.open_notification_settings();
		}
	}

	fn voice_advanced(&mut self, ui: &mut egui::Ui, active: bool) {
		let colors = design::palette(ui);
		let confirm = egui::Id::unique("voice-reset-confirm");
		design::card(ui, |ui| {
			if design::row(
				ui,
				"voice-reset-settings",
				Some("voice-reset-settings-description"),
				|ui| {
					design::button(
						ui,
						&crate::i18n::translate("voice-reset"),
						design::ButtonKind::Outline,
					)
				},
			)
			.clicked()
			{
				ui.data_mut(|data| data.insert_temp(confirm, true));
			}
			if active && let Some(code) = &self.voice_privacy_code {
				design::card_divider(ui);
				egui::CollapsingHeader::new(crate::i18n::translate(
					"voice-voice-settings-content-voice-privacy-code",
				))
				.show(ui, |ui| {
					ui.add(
						egui::Label::new(RichText::new(code).monospace())
							.selectable(true)
							.wrap(),
					);
					ui.label(
						RichText::new(crate::i18n::translate(
							"voice-voice-settings-content-compare-with-the-other-participants-this-code-changes-with-the",
						))
						.size(12.0)
						.color(colors.muted),
					);
				});
			}
		});
		if ui.data(|data| data.get_temp::<bool>(confirm).unwrap_or(false))
			&& let Some(choice) = crate::dialog::Confirm::new(
				"voice-reset-confirm",
				"voice-reset-confirm-title",
				"voice-reset-confirm-message",
			)
			.confirm_label("voice-reset")
			.show(ui.ctx())
		{
			if choice == crate::dialog::Choice::Confirmed {
				self.reset_voice_settings();
			}
			ui.data_mut(|data| data.remove::<bool>(confirm));
		}
	}

	/// Returns devices, volumes, input mode, processing and camera to their defaults.
	fn reset_voice_settings(&mut self) {
		self.voice_input = None;
		self.voice_output = None;
		self.voice_gain = crate::VoiceGain::default();
		self.voice_push_to_talk = false;
		self.voice_processing = Default::default();
		self.voice_camera_device = None;
		self.voice_camera_rate = Default::default();
	}
}

/// One switch and preview link per cue.
pub(super) fn sound_rows(
	ui: &mut egui::Ui,
	sounds: Vec<(&str, &mut bool, Sound)>,
	preview: &mut Option<Sound>,
) {
	for (index, (label, value, sound)) in sounds.into_iter().enumerate() {
		if index > 0 {
			design::card_divider(ui);
		}
		design::switch(ui, label, None, value);
		if design::text_action(
			ui,
			&crate::i18n::translate("notification-settings-notification-settings-preview-sound"),
		)
		.clicked()
		{
			*preview = Some(sound);
		}
	}
}

/// Card that opens another settings page.
fn related_link(ui: &mut egui::Ui, icon: Icon, title: &str, detail: &str) -> egui::Response {
	let colors = design::palette(ui);
	design::card(ui, |ui| {
		ui.horizontal(|ui| {
			let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
			ui.painter().rect_filled(rect, 8.0, colors.hover);
			crate::icons::paint(ui.painter(), icon, rect.shrink(9.0), colors.text);
			let title = crate::i18n::translate(title);
			design::row(ui, &title, Some(detail), |ui| {
				crate::icons::button(ui, Icon::ChevronRight, 32.0, &title)
			})
		})
		.inner
	})
}

/// Loading and failure states of the account's voice settings, which every control here edits.
fn settings_status(ui: &mut egui::Ui, state: &mut State, commands: &mut Vec<client_core::Command>) {
	if let Some(error) = state.soundboard.error {
		design::notice(ui, design::Level::Error, error.label());
		if design::text_action(ui, &crate::i18n::translate("account-retry")).clicked() {
			commands.extend(state.read_soundboard_settings());
		}
	} else if state.soundboard.settings().is_none() {
		design::hint(ui, &crate::i18n::translate("voice-settings-loading"));
	}
}

fn stream_settings(ui: &mut egui::Ui, state: &mut State, commands: &mut Vec<client_core::Command>) {
	let settings = state.soundboard.settings();
	let mut previews = settings.is_none_or(|settings| settings.stream_previews);
	let ready = settings.is_some();
	design::card(ui, |ui| {
		ui.add_enabled_ui(ready, |ui| {
			if design::switch(
				ui,
				"voice-stream-previews",
				Some("voice-stream-previews-description"),
				&mut previews,
			)
			.changed()
			{
				commands.extend(state.save_soundboard(Edit::StreamPreviews(previews)));
			}
		});
		settings_status(ui, state, commands);
	});
}

fn soundboard_settings(
	ui: &mut egui::Ui,
	state: &mut State,
	commands: &mut Vec<client_core::Command>,
) {
	let colors = design::palette(ui);
	let Some(settings) = state.soundboard.settings().cloned() else {
		design::card(ui, |ui| settings_status(ui, state, commands));
		return;
	};
	design::card(ui, |ui| {
		let id = egui::Id::unique("soundboard-volume");
		let mut volume = ui
			.data(|data| data.get_temp::<f32>(id))
			.unwrap_or(settings.volume);
		let response = design::slider_row(
			ui,
			"voice-soundboard-volume",
			Some("voice-soundboard-volume-description"),
			&mut volume,
			0.0..=100.0,
			"%",
		);
		if response.dragged() {
			ui.data_mut(|data| data.insert_temp(id, volume));
		} else {
			ui.data_mut(|data| data.remove::<f32>(id));
			if response.changed() || response.drag_stopped() {
				commands.extend(state.save_soundboard(Edit::Volume(volume)));
			}
		}
		settings_status(ui, state, commands);
	});
	ui.add_space(8.0);
	ui.label(
		design::semibold(ui, crate::i18n::translate("voice-entrance-sounds"), 16.0)
			.color(colors.text_strong),
	);
	ui.add(
		egui::Label::new(
			RichText::new(crate::i18n::translate("voice-entrance-sounds-description"))
				.size(13.0)
				.color(colors.muted),
		)
		.wrap(),
	);
	let nitro = state.premium_type != 0 || state.demo;
	let scope_id = egui::Id::unique("entrance-scope");
	let mut scope = Id(ui
		.data(|data| data.get_temp::<u64>(scope_id))
		.unwrap_or(ALL_SERVERS.0));
	if scope != ALL_SERVERS && !state.guilds.iter().any(|guild| guild.id == scope) {
		scope = ALL_SERVERS;
	}
	let scope_name = |scope: Id| {
		if scope == ALL_SERVERS {
			crate::i18n::translate("voice-entrance-sounds-all-servers")
		} else {
			state
				.guilds
				.iter()
				.find(|guild| guild.id == scope)
				.map_or_else(String::new, |guild| guild.name.clone())
		}
	};
	ui.label(
		design::semibold(
			ui,
			crate::i18n::translate("voice-entrance-sounds-server"),
			13.0,
		)
		.color(colors.text_strong),
	);
	let mut chosen = scope;
	design::Select::from_id_salt("entrance-scope")
		.selected_text(scope_name(scope))
		.width(ui.available_width())
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut chosen, ALL_SERVERS, scope_name(ALL_SERVERS));
			for guild in &state.guilds {
				ui.selectable_value(&mut chosen, guild.id, &guild.name);
			}
		});
	if chosen != scope {
		ui.data_mut(|data| data.insert_temp(scope_id, chosen.0));
		scope = chosen;
	}
	ui.add_space(4.0);
	ui.label(
		design::semibold(
			ui,
			crate::i18n::translate("voice-entrance-sounds-sound"),
			13.0,
		)
		.color(colors.text_strong),
	);
	if !nitro {
		design::hint(ui, &crate::i18n::translate("voice-entrance-sounds-nitro"));
	}
	let current = settings
		.join_sounds
		.iter()
		.find(|(key, _)| *key == scope)
		.map(|(_, sound)| *sound);
	let mut edit = None;
	ui.add_enabled_ui(nitro, |ui| {
		let picker = match current {
			Some(sound) => {
				if state.soundboard.sound(sound.sound).is_none() {
					commands.extend(state.read_soundboard_sounds(sound.guild));
				}
				let name = state
					.soundboard
					.sound(sound.sound)
					.map_or_else(|| "\u{2026}".to_owned(), sound_label);
				design::card(ui, |ui| {
					ui.horizontal(|ui| {
						ui.label(RichText::new(name).size(14.0).color(colors.text_strong));
						let picker = crate::icons::button(
							ui,
							Icon::Pencil,
							28.0,
							&crate::i18n::translate("voice-entrance-sounds-edit"),
						);
						if crate::icons::button(
							ui,
							Icon::Trash,
							28.0,
							&crate::i18n::translate("voice-entrance-sounds-remove"),
						)
						.clicked()
						{
							edit = Some(None);
						}
						picker
					})
					.inner
				})
			}
			None => design::button(
				ui,
				&crate::i18n::translate("voice-entrance-sounds-sound"),
				design::ButtonKind::Outline,
			),
		};
		let mut open = ui
			.data(|data| data.get_temp::<bool>(picker.id))
			.unwrap_or(false);
		if picker.clicked() {
			open = !open;
		}
		egui::Popup::menu(&picker)
			.open_bool(&mut open)
			.width(320.0)
			.close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
			.show(|ui| {
				egui::ScrollArea::vertical()
					.max_height(360.0)
					.show(ui, |ui| {
						if let Some(sound) = sound_picker(ui, state, commands, scope) {
							edit = Some(Some(sound));
						}
					});
			});
		if edit.is_some() {
			open = false;
		}
		ui.data_mut(|data| data.insert_temp(picker.id, open));
	});
	if let Some(sound) = edit {
		commands.extend(state.save_soundboard(Edit::JoinSound { scope, sound }));
	}
}

/// Default sounds first, then each server's sounds, read when its group is opened.
fn sound_picker(
	ui: &mut egui::Ui,
	state: &mut State,
	commands: &mut Vec<client_core::Command>,
	scope: Id,
) -> Option<JoinSound> {
	let mut groups = vec![(
		None,
		crate::i18n::translate("voice-soundboard-default-sounds"),
	)];
	groups.extend(
		state
			.guilds
			.iter()
			.map(|guild| (Some(guild.id), guild.name.clone())),
	);
	let mut picked = None;
	for (guild, name) in groups {
		egui::CollapsingHeader::new(name)
			.id_salt(("soundboard-group", guild))
			.default_open(guild.is_none() || guild == Some(scope))
			.show(ui, |ui| {
				commands.extend(state.read_soundboard_sounds(guild));
				match state.soundboard.sounds(guild) {
					None => {
						ui.spinner();
					}
					Some([]) => {
						ui.weak(crate::i18n::translate("voice-soundboard-no-sounds"));
					}
					Some(sounds) => {
						for sound in sounds.iter().filter(|sound| sound.available) {
							if ui.selectable_label(false, sound_label(sound)).clicked() {
								picked = Some(JoinSound {
									sound: sound.id,
									guild: sound.guild,
								});
							}
						}
					}
				}
			});
	}
	picked
}

fn sound_label(sound: &model::soundboard::Sound) -> String {
	match &sound.emoji_name {
		Some(emoji) => format!("{emoji} {}", sound.name),
		None => sound.name.clone(),
	}
}
