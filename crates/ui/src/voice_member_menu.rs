//! Discord's context menu on a voice channel member: profile and note, this device's volume
//! and mutes, roles and moving, then a moderator's server mute, server deafen, disconnect and
//! removal.
use crate::{MessagingUi, design, icons::Icon, user_menu::Action};
use client_core::{State, auth::AuthState, voice::RosterEntry};
use model::{Id, server_admin::VoiceChange};

/// Participants kept in each device-local list (soundboard mutes, hidden cameras).
const MAX_LOCAL: usize = 64;

/// A server nickname being edited from a member menu.
pub(crate) struct NicknameEdit {
	guild: Id,
	user: Id,
	name: String,
	submitted: bool,
}

impl MessagingUi {
	pub(super) fn voice_participant_menu(
		&mut self,
		response: &egui::Response,
		state: &State,
		entry: &RosterEntry,
	) {
		crate::user_menu::popup(response, egui::Popup::default_response_id(response))
			.close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
			.show(|ui| {
				ui.set_width(220.0);
				ui.spacing_mut().button_padding = egui::vec2(8.0, 6.0);
				self.voice_member_menu(ui, state, entry);
			});
	}

	fn voice_member_menu(&mut self, ui: &mut egui::Ui, state: &State, entry: &RosterEntry) {
		let colors = design::palette(ui);
		let target = entry.participant.user;
		let channel = entry.channel;
		let own = state.user.as_ref().is_some_and(|own| own.id == target);
		let user = crate::voice::resolve_member(state, entry).0.cloned();
		let inactive = crate::voice::inactive_recipient(state, channel, target);
		let guild = state
			.channel(channel)
			.and_then(|channel| channel.guild)
			.filter(|guild| *guild == entry.guild);
		let online =
			state.demo || (state.gateway_connected && state.auth == AuthState::Authenticated);
		self.ring_item(ui, state, entry);

		if let Some(user) = &user {
			if ui
				.button(crate::i18n::translate("user-menu-contents-profile"))
				.clicked()
			{
				self.profile.command_open(user.clone());
				ui.close();
			}
			let mention = state.selected.is_some_and(|id| {
				state
					.channel(id)
					.is_some_and(|channel| channel.supports_text())
			});
			if mention
				&& ui
					.button(crate::i18n::translate("user-menu-contents-mention"))
					.clicked()
			{
				self.user_action = Some(Action::Mention(user.clone()));
				ui.close();
			}
			if !own {
				if ui
					.add_enabled(
						state.can_open_user_dm(user),
						egui::Button::new(crate::i18n::translate("friends-message")),
					)
					.clicked()
				{
					self.user_action = Some(Action::Message(user.clone()));
					ui.close();
				}
				let mut note = egui::text::LayoutJob::default();
				for (text, size, color) in [
					(
						crate::i18n::translate("user-menu-contents-add-note"),
						14.0,
						colors.text,
					),
					(
						format!("\n{}", crate::i18n::translate("voice-member-note-hint")),
						12.0,
						colors.muted,
					),
				] {
					note.append(
						&text,
						0.0,
						egui::TextFormat::simple(egui::FontId::proportional(size), color),
					);
				}
				if ui
					.add_enabled(
						online && !state.user_action_pending(),
						egui::Button::new(note),
					)
					.clicked()
				{
					self.user_action = Some(Action::Note(user.clone()));
					ui.close();
				}
			}
		}

		if !own && !inactive {
			ui.separator();
			let mut volume = self
				.voice_user_volumes
				.as_deref()
				.and_then(|values| values.iter().find(|(user, _)| *user == target.0))
				.map_or(100, |(_, volume)| *volume);
			if crate::voice::gain_slider(
				ui,
				&mut volume,
				"voice-voice-participant-menu-user-volume",
			)
			.changed()
			{
				self.set_voice_user_volume(target, volume);
			}
		}

		ui.separator();
		if own {
			let muted = self.voice_muted || self.voice_deafened;
			if check_row(ui, "voice-member-mute", muted, false).clicked() {
				self.voice_self_toggle = Some(false);
				ui.close();
			}
			if check_row(ui, "voice-member-deafen", self.voice_deafened, false).clicked() {
				self.voice_self_toggle = Some(true);
				ui.close();
			}
		} else if !inactive {
			let muted = self.voice_user_locally_muted(target);
			if check_row(ui, "voice-member-mute", muted, false).clicked() {
				self.set_voice_user_locally_muted(target, !muted);
			}
			for (key, list) in [
				(
					"voice-member-mute-soundboard",
					&mut self.voice_soundboard_muted,
				),
				("voice-member-disable-video", &mut self.voice_video_disabled),
			] {
				let on = list.contains(&target.0);
				if check_row(ui, key, on, false).clicked() {
					list.retain(|id| *id != target.0);
					if !on && list.len() < MAX_LOCAL {
						list.push(target.0);
					}
				}
			}
			let encrypted = self.voice_privacy_code.is_some()
				&& state.voice.active.as_ref().is_some_and(|call| {
					call.channel == channel && call.participants.iter().any(|p| p.user == target)
				});
			if encrypted
				&& ui
					.button(crate::i18n::translate(
						"voice-member-view-verification-code",
					))
					.clicked()
			{
				self.voice_verify = Some(target);
				self.voice_verification_code = None;
				ui.close();
			}
		}
		if let Some(guild) = guild
			&& !own && user.is_some()
			&& state.can_edit_guild_nickname(guild, target)
			&& ui
				.add_enabled(
					online && !state.server_admin.pending,
					egui::Button::new(crate::i18n::translate("voice-member-change-nickname")),
				)
				.clicked()
		{
			self.member_nickname = Some(NicknameEdit {
				guild,
				user: target,
				name: entry
					.member
					.as_ref()
					.and_then(|member| member.nick.clone())
					.unwrap_or_default(),
				submitted: false,
			});
			ui.close();
		}
		if let Some(user) = user.as_ref().filter(|user| !user.webhook) {
			crate::extensions_ui::context_menu_targets(
				ui,
				&[extensions::MenuTarget::User {
					user_id: user.id.to_string(),
					guild_id: guild.map(|guild| guild.to_string()),
					channel_id: Some(channel.to_string()),
				}],
			);
		}
		if let Some(user) = user.as_ref().filter(|_| !own) {
			let enabled = online && !state.user_action_pending();
			if let Some(ignore) = crate::profiles::ignore_button(ui, state, user, enabled) {
				self.user_action = Some(ignore);
			}
			let blocked = state.user_blocked(user.id) == Some(true);
			if ui
				.add_enabled(
					enabled,
					egui::Button::new(
						egui::RichText::new(crate::i18n::translate(if blocked {
							"user-menu-contents-unblock"
						} else {
							"user-menu-contents-block"
						}))
						.color(colors.danger),
					),
				)
				.clicked()
			{
				self.user_action = Some(Action::Block {
					user: user.id,
					blocked: !blocked,
				});
				ui.close();
			}
		}

		let Some(guild) = guild else {
			copy_id(ui, target);
			return;
		};
		let destinations: Vec<_> = state
			.channels
			.iter()
			.filter(|destination| {
				state.can_moderate_voice(
					guild,
					channel,
					target,
					VoiceChange::Move(Some(destination.id)),
				)
			})
			.map(|destination| (destination.id, destination.name.clone()))
			.collect();
		let roles = !state.member_menu_roles(guild, target).is_empty();
		let movable = !destinations.is_empty()
			|| state.can_moderate_voice(guild, channel, target, VoiceChange::Move(None));
		if roles || movable {
			ui.separator();
			crate::user_menu::roles_menu(ui, state, guild, target, &mut self.user_action);
			if movable {
				ui.add_enabled_ui(online && !state.server_admin.pending, |ui| {
					ui.menu_button(crate::i18n::translate("voice-member-move-to"), |ui| {
						egui::ScrollArea::vertical()
							.max_height(240.0)
							.show(ui, |ui| {
								for (id, name) in &destinations {
									if ui.button(name).clicked() {
										self.voice_moderation = Some((
											guild,
											channel,
											target,
											VoiceChange::Move(Some(*id)),
										));
										ui.close();
									}
								}
							});
					});
				});
			}
		}

		let server_muted = entry.participant.server_muted;
		let server_deafened = entry.participant.server_deafened;
		let changes = [
			(
				"voice-member-server-mute",
				Some(server_muted),
				VoiceChange::Mute(!server_muted),
			),
			(
				"voice-member-server-deafen",
				Some(server_deafened),
				VoiceChange::Deafen(!server_deafened),
			),
			("voice-member-disconnect", None, VoiceChange::Move(None)),
		]
		.into_iter()
		.filter(|(_, _, change)| state.can_moderate_voice(guild, channel, target, *change))
		.collect::<Vec<_>>();
		let removals = user
			.as_ref()
			.filter(|_| !own)
			.map(|user| crate::user_menu::moderation_kinds(state, guild, user))
			.unwrap_or_default();
		if !changes.is_empty() || !removals.is_empty() {
			ui.separator();
			ui.add_enabled_ui(online && !state.server_admin.pending, |ui| {
				for (key, checked, change) in changes {
					let clicked = match checked {
						Some(checked) => check_row(ui, key, checked, true).clicked(),
						None => ui
							.button(
								egui::RichText::new(crate::i18n::translate(key))
									.color(colors.danger),
							)
							.clicked(),
					};
					if clicked {
						self.voice_moderation = Some((guild, channel, target, change));
						ui.close();
					}
				}
			});
			if let Some(user) = &user {
				crate::user_menu::moderation_items(
					ui,
					guild,
					user,
					&removals,
					&mut self.user_action,
				);
			}
		}
		copy_id(ui, target);
	}

	/// Rings a private call recipient who has not joined yet.
	fn ring_item(&mut self, ui: &mut egui::Ui, state: &State, entry: &RosterEntry) {
		let user = entry.participant.user;
		let Some(call) = state
			.voice
			.active
			.as_ref()
			.filter(|call| call.channel == entry.channel && call.guild.is_none())
			.filter(|call| !call.participants.iter().any(|p| p.user == user))
		else {
			return;
		};
		if state.user.as_ref().is_none_or(|own| own.id == user) {
			return;
		}
		let stop = state.voice.ringing(call.channel).contains(&user);
		let enabled = state
			.ring_recipient(call.channel, call.request, user, stop)
			.is_some();
		if ui
			.add_enabled(
				enabled,
				egui::Button::new(crate::i18n::translate(if stop {
					"voice-recipient-stop-ringing"
				} else {
					"voice-recipient-ring"
				})),
			)
			.clicked()
		{
			self.ring_request = Some((call.channel, call.request, user, stop));
			ui.close();
		}
		ui.separator();
	}

	/// Applies own mute or deafen chosen in the member menu.
	pub(super) fn apply_voice_self_toggle(
		&mut self,
		state: &mut State,
		commands: &mut Vec<client_core::Command>,
	) {
		if let Some(deafen) = self.voice_self_toggle.take() {
			let (muted, deafened) = self.toggle_voice(deafen);
			if let Some(command) = state.set_call_mute(muted, deafened) {
				commands.push(command);
			}
		}
	}

	/// The pairwise end-to-end verification code both participants compare out of band.
	pub(super) fn verification_code_dialog(&mut self, ctx: &egui::Context, state: &State) {
		let Some(user) = self.voice_verify else {
			return;
		};
		let call = state
			.voice
			.active
			.as_ref()
			.filter(|call| call.participants.iter().any(|p| p.user == user));
		let Some(call) = call else {
			self.voice_verify = None;
			self.voice_verification_code = None;
			return;
		};
		let name = crate::voice::participant_user(state, call.channel, user)
			.map_or_else(String::new, |user| user.name.clone());
		let code = self
			.voice_verification_code
			.as_ref()
			.filter(|(id, _)| *id == user && self.voice_privacy_code.is_some())
			.map(|(_, code)| code.as_deref());
		let mut close = false;
		let response = crate::dialog::Dialog::new(
			"voice-verification-code",
			crate::i18n::translate_args("voice-member-verification-title", &[("name", &name)]),
		)
		.subtitle(crate::i18n::translate(
			"voice-member-verification-description",
		))
		.icon(Icon::Lock)
		.width(420.0)
		.show(ctx, |d| {
			d.content(|ui| match code {
				Some(Some(code)) => {
					let colors = design::palette(ui);
					egui::Grid::new("voice-verification-groups")
						.spacing(egui::vec2(18.0, 10.0))
						.show(ui, |ui| {
							for (index, group) in code.as_bytes().chunks(5).enumerate() {
								ui.label(
									egui::RichText::new(String::from_utf8_lossy(group))
										.monospace()
										.size(20.0)
										.color(colors.text_strong),
								);
								if index % 3 == 2 {
									ui.end_row();
								}
							}
						});
				}
				Some(None) => crate::dialog::notice(
					ui,
					crate::dialog::Level::Warning,
					"voice-member-verification-unavailable",
				),
				None => crate::dialog::hint(ui, "voice-member-verification-loading"),
			});
			d.footer(|ui| {
				close |= crate::dialog::action(
					ui,
					"voice-member-verification-done",
					crate::dialog::Action::Primary,
				)
				.clicked();
			});
		});
		if close || response.close {
			self.voice_verify = None;
			self.voice_verification_code = None;
		}
	}

	/// Discord's Change Nickname dialog for another member of the server.
	pub(super) fn member_nickname_dialog(
		&mut self,
		ctx: &egui::Context,
		state: &mut State,
		commands: &mut Vec<client_core::Command>,
	) {
		let Some(edit) = &mut self.member_nickname else {
			return;
		};
		if edit.submitted && !state.server_admin.pending {
			if state.server_admin.error.is_none() {
				self.member_nickname = None;
				return;
			}
			edit.submitted = false;
		}
		let ready = !state.server_admin.pending
			&& state.can_edit_guild_nickname(edit.guild, edit.user)
			&& !edit.name.chars().any(char::is_control);
		let mut save = false;
		let mut close = false;
		let response = crate::dialog::Dialog::new(
			"member-nickname",
			crate::i18n::translate("voice-member-change-nickname"),
		)
		.width(440.0)
		.show(ctx, |d| {
			d.content(|ui| {
				ui.spacing_mut().item_spacing.y = 10.0;
				let label = crate::dialog::label(ui, "server-admin-dialog-nickname");
				let input = crate::dialog::input(
					ui,
					egui::TextEdit::singleline(&mut edit.name)
						.align(egui::Align2::LEFT_CENTER)
						.hint_text(crate::i18n::translate(
							"server-admin-dialog-use-their-username",
						))
						.char_limit(32),
				)
				.labelled_by(label.id);
				save |= ready
					&& input.lost_focus()
					&& ui.input(|input| input.key_pressed(egui::Key::Enter));
				crate::dialog::hint(ui, "server-admin-dialog-leave-blank-to-use-their-username");
				if let Some(error) = state.server_admin.error {
					crate::dialog::notice(ui, crate::dialog::Level::Error, error);
				}
			});
			d.footer(|ui| {
				ui.add_enabled_ui(ready, |ui| {
					save |= crate::dialog::action(
						ui,
						"server-admin-dialog-save",
						crate::dialog::Action::Primary,
					)
					.clicked();
				});
				close |= crate::dialog::action(
					ui,
					"server-admin-dialog-cancel",
					crate::dialog::Action::Neutral,
				)
				.clicked();
			});
		});
		if save
			&& let Some(command) = state.request_server_admin(
				edit.guild,
				model::server_admin::Action::SetNickname {
					user: edit.user,
					nick: edit.name.trim().to_owned(),
				},
			) {
			commands.push(command);
			edit.submitted = true;
		}
		if close || (response.close && !edit.submitted) {
			self.member_nickname = None;
		}
	}
}

/// Menu row with Discord's trailing checkbox; moderator rows are red.
fn check_row(ui: &mut egui::Ui, key: &str, checked: bool, danger: bool) -> egui::Response {
	let colors = design::palette(ui);
	let label = crate::i18n::translate(key);
	let (rect, response) =
		ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::CheckBox, ui.is_enabled(), checked, &label)
	});
	let enabled = ui.is_enabled();
	if enabled && (response.hovered() || response.has_focus()) {
		ui.painter().rect_filled(rect, 4.0, colors.hover);
	}
	let mut color = if danger { colors.danger } else { colors.text };
	if !enabled {
		color = color.gamma_multiply(0.5);
	}
	ui.painter().text(
		rect.left_center() + egui::vec2(8.0, 0.0),
		egui::Align2::LEFT_CENTER,
		&label,
		egui::FontId::proportional(14.0),
		color,
	);
	let square = egui::Rect::from_center_size(
		rect.right_center() - egui::vec2(18.0, 0.0),
		egui::Vec2::splat(18.0),
	);
	if checked {
		ui.painter().rect_filled(square, 4.0, colors.accent);
		crate::icons::paint(
			ui.painter(),
			Icon::Check,
			square.shrink(3.0),
			colors.accent_text,
		);
	} else {
		ui.painter().rect_stroke(
			square,
			4.0,
			egui::Stroke::new(1.5, colors.muted),
			egui::StrokeKind::Inside,
		);
	}
	response
}

fn copy_id(ui: &mut egui::Ui, user: Id) {
	if !crate::developer_mode() {
		return;
	}
	ui.separator();
	if ui
		.button(crate::i18n::translate(
			"server-admin-member-menu-copy-user-id",
		))
		.clicked()
	{
		ui.ctx().copy_text(user.to_string());
		ui.close();
	}
}
