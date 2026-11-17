//! User settings modal in Discord's layout: a sidebar with the profile header, search and grouped
//! pages on the left, and the selected page under its title and close button on the right.
use crate::{MessagingUi, design, i18n::Language, icons};
use client_core::State;
use egui::RichText;

#[derive(Default)]
pub(super) struct Settings {
	pub open: bool,
	page: Page,
	query: String,
	pub(super) sections: SectionNav,
	language_time_format: Option<(model::TimeFormat, Language)>,
	pub(super) editor: crate::profile_edit::Editor,
	pub(super) notifications: crate::notification_settings::Navigation,
	pub(super) messaging_permissions: crate::messaging_permissions::Navigation,
	pub(super) games: crate::registered_games::Page,
}

/// The open page's sections: the one at the top of the view and a pending sidebar jump.
#[derive(Default)]
pub(super) struct SectionNav {
	page: Page,
	active: usize,
	jump: Option<usize>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Page {
	Account,
	Profile,
	Storage,
	MessagingPermissions,
	Notifications,
	Nitro,
	Boost,
	Subscriptions,
	Gifts,
	Billing,
	Voice,
	#[default]
	Appearance,
	Accessibility,
	/// Discord's System page: startup and window behaviour, keybinds and graphics.
	General,
	/// Display language and clock style.
	LanguageTime,
	/// Discord's Registered Games: the current game and Added Games.
	Activity,
	ActivityPrivacy,
	/// Discord's Connections and Authorized Apps.
	Connections,
	Extensions,
	Themes,
	Updates,
	Developer,
}
impl Page {
	/// Every page in sidebar order; the narrow-window page picker lists them the same way.
	const ALL: [Self; 22] = [
		Self::Account,
		Self::Profile,
		Self::Storage,
		Self::MessagingPermissions,
		Self::Notifications,
		Self::Nitro,
		Self::Boost,
		Self::Subscriptions,
		Self::Gifts,
		Self::Billing,
		Self::Voice,
		Self::Appearance,
		Self::Accessibility,
		Self::General,
		Self::LanguageTime,
		Self::Activity,
		Self::ActivityPrivacy,
		Self::Connections,
		Self::Extensions,
		Self::Themes,
		Self::Updates,
		Self::Developer,
	];
	/// Sidebar groups in Discord's order, each with an optional heading. Profile opens from the
	/// header above the groups; Serein's own pages sit in their own group before Developer.
	const SECTIONS: [(Option<&'static str>, &'static [Self]); 6] = [
		(
			None,
			&[
				Self::Account,
				Self::Storage,
				Self::MessagingPermissions,
				Self::Notifications,
			],
		),
		(
			Some("section-billing"),
			&[
				Self::Nitro,
				Self::Boost,
				Self::Subscriptions,
				Self::Gifts,
				Self::Billing,
			],
		),
		(
			Some("section-experience"),
			&[
				Self::Voice,
				Self::Appearance,
				Self::Accessibility,
				Self::General,
				Self::LanguageTime,
			],
		),
		(
			Some("section-games-apps"),
			&[Self::Activity, Self::ActivityPrivacy, Self::Connections],
		),
		(
			Some("section-serein"),
			&[Self::Extensions, Self::Themes, Self::Updates],
		),
		(None, &[Self::Developer]),
	];
	/// Section headings in page order; two or more appear as sidebar sub-items.
	fn sections(self) -> &'static [&'static str] {
		match self {
			Self::Storage => &["settings-sub-local-data", "settings-sub-voice-e2ee"],
			Self::Voice => &[
				"settings-sub-voice",
				"settings-sub-camera",
				"settings-sub-streaming",
				"settings-sub-sounds",
				"settings-sub-soundboard",
				"settings-sub-advanced",
			],
			Self::Appearance => &[
				"settings-sub-theme",
				"settings-sub-messages",
				"settings-sub-chat-box",
				"settings-sub-streamer-mode",
				"settings-sub-advanced",
			],
			Self::Accessibility => &["settings-sub-visual-density", "settings-sub-reduced-motion"],
			Self::General => &[
				"page-general",
				"settings-sub-custom-keybinds",
				"settings-sub-default-keybinds",
				"settings-sub-advanced",
			],
			Self::Activity => &[
				"settings-activity-current-game",
				"settings-activity-added-games",
			],
			Self::ActivityPrivacy => &["settings-sub-shared-activity"],
			Self::Connections => &["settings-sub-connections", "settings-sub-authorized-apps"],
			Self::Billing => &["settings-sub-payment-methods", "settings-sub-transactions"],
			Self::Subscriptions => &["settings-sub-your-subscriptions"],
			Self::Boost => &["settings-sub-active-boosts"],
			Self::Gifts => &["settings-sub-purchased-gifts"],
			_ => &[],
		}
	}
	/// Sidebar row label, also the page title.
	fn row_label(self, language: Language) -> String {
		match self {
			Self::General => language.text("page-system"),
			page => page.label(language),
		}
	}
	fn icon(self) -> icons::Icon {
		match self {
			Self::Account | Self::Profile => icons::Icon::User,
			Self::Storage => icons::Icon::ShieldWarning,
			Self::MessagingPermissions => icons::Icon::ChatDots,
			Self::Notifications => icons::Icon::Bell,
			Self::Nitro => icons::Icon::Nitro,
			Self::Boost => icons::Icon::Boost,
			Self::Subscriptions => icons::Icon::Subscriptions,
			Self::Gifts => icons::Icon::Gift,
			Self::Billing => icons::Icon::CreditCard,
			Self::Voice => icons::Icon::Microphone,
			Self::Appearance => icons::Icon::Palette,
			Self::Accessibility => icons::Icon::Accessibility,
			Self::General => icons::Icon::Keyboard,
			Self::LanguageTime => icons::Icon::Translate,
			Self::Activity => icons::Icon::GameController,
			Self::ActivityPrivacy => icons::Icon::ActivityPrivacy,
			Self::Connections => icons::Icon::Link,
			Self::Extensions => icons::Icon::Puzzle,
			Self::Themes => icons::Icon::PaintBrush,
			Self::Updates => icons::Icon::Download,
			Self::Developer => icons::Icon::Code,
		}
	}
	fn label_key(self) -> &'static str {
		match self {
			Self::Account => "page-account",
			Self::Profile => "page-profile",
			Self::Storage => "page-storage",
			Self::MessagingPermissions => "page-messaging-permissions",
			Self::Notifications => "page-notifications",
			Self::Nitro => "page-nitro",
			Self::Boost => "page-boost",
			Self::Subscriptions => "page-subscriptions",
			Self::Gifts => "page-gifts",
			Self::Billing => "page-billing",
			Self::Voice => "page-voice",
			Self::Appearance => "page-appearance",
			Self::Accessibility => "page-accessibility",
			Self::General => "page-general",
			Self::LanguageTime => "page-language-time",
			Self::Activity => "page-registered-games",
			Self::ActivityPrivacy => "page-activity-privacy",
			Self::Connections => "page-connections",
			Self::Extensions => "page-extensions",
			Self::Themes => "page-themes",
			Self::Updates => "page-updates",
			Self::Developer => "page-developer",
		}
	}
	fn label(self, language: Language) -> String {
		language.text(self.label_key())
	}
	fn matches(self, query: &str, language: Language) -> bool {
		let keywords = match self {
			Self::Account => "my account profile logout",
			Self::Profile => "profile edit display name about me bio pronouns color colour",
			Self::Storage => {
				"data privacy local storage clear cache drafts credentials voice end-to-end encryption e2ee persistent verification codes"
			}
			Self::MessagingPermissions => {
				"messaging permissions spam filters direct messages dm friend requests personalized connected games"
			}
			Self::Notifications => {
				"notifications desktop system alerts overview sounds badges message ring idle away afk timeout lock locked phone mobile push"
			}
			Self::Nitro => "nitro premium perks",
			Self::Boost => "server boost boosts boosting premium guild",
			Self::Subscriptions => "subscriptions subscription nitro renew billing",
			Self::Gifts => "gift inventory gifts nitro",
			Self::Billing => "billing payment methods card transactions history",
			Self::Voice => {
				"voice video camera preview audio microphone speakers devices volume gain noise suppression push to talk streaming stream preview sounds deafen mute soundboard entrance sounds reset"
			}
			Self::Appearance => {
				"appearance streamer mode obs xsplit customization font typography import ttf otf primary accent hex window effects transparency blur gradient names cosmetics decorations theme dark light system mode layout sidebar width people members member list reset colour color preset chat messages media links confirm confirmation external browser smooth scrolling scroll speed trackpad wheel hidden channels channel list emoji emojis nitro fake suggestions autocomplete locked emoticons chat box automatically convert"
			}
			Self::Accessibility => {
				"accessibility zoom scale density compact spacing reduced motion animate animated gifs autoplay"
			}
			Self::General => {
				"system general windows macos linux login menu bar startup autostart automatically open minimized minimize close tray background title bar caption window buttons decorations borderless tiling graphics gpu adapter render discrete integrated hardware acceleration performance battery vrr variable refresh rate steady refresh rate g-sync gsync freesync adaptive sync flicker flickering steady frame rate fps vsync keybinds keyboard shortcuts custom default formatting navigation"
			}
			Self::LanguageTime => {
				"language time clock 12 hour 24 hour am pm format locale translation"
			}
			Self::Activity => {
				"game activity playing osu registered games added current game detection detected process program executable rename wrong add hide last played"
			}
			Self::ActivityPrivacy => "activity privacy share sharing status presence game",
			Self::Connections => "connections connected accounts authorized apps oauth deauthorize",
			Self::Extensions => "extensions plugins shop store catalog import community tools",
			Self::Themes => "themes shop store catalog import community appearance colors",
			Self::Updates => {
				"updates auto update release channel production stable nightly download restart version check diagnostics issue bug system info debug"
			}
			Self::Developer => "developer mode copy id ids",
		};
		keywords.contains(query)
			|| self.label(language).to_lowercase().contains(query)
			|| self.row_label(language).to_lowercase().contains(query)
	}
}

fn gpu_label(language: Language, preference: model::GpuPreference) -> String {
	language.text(match preference {
		model::GpuPreference::Automatic => "gpu-automatic",
		model::GpuPreference::HighPerformance => "gpu-high-performance",
		model::GpuPreference::PowerSaving => "gpu-power-saving",
	})
}

fn graphics_api_label(language: Language, api: model::GraphicsApi) -> String {
	if api == model::GraphicsApi::Automatic {
		language.text("gpu-automatic")
	} else {
		api.label().to_owned()
	}
}

fn gpu_description(language: Language, preference: model::GpuPreference) -> String {
	language.text(match preference {
		model::GpuPreference::Automatic => "gpu-automatic-description",
		model::GpuPreference::HighPerformance => "gpu-high-performance-description",
		model::GpuPreference::PowerSaving => "gpu-power-saving-description",
	})
}

impl MessagingUi {
	pub(super) fn open_extension_settings(&mut self, view: extensions::AppView) {
		self.settings.page = match view {
			extensions::AppView::Settings => Page::General,
			extensions::AppView::Account => Page::Account,
			extensions::AppView::ProfileSettings => Page::Profile,
			extensions::AppView::Appearance => Page::Appearance,
			extensions::AppView::MessagingPermissions => Page::MessagingPermissions,
			extensions::AppView::Notifications => Page::Notifications,
			extensions::AppView::Activity => Page::Activity,
			extensions::AppView::Extensions => Page::Extensions,
			extensions::AppView::Themes => Page::Themes,
			extensions::AppView::VoiceSettings => Page::Voice,
			extensions::AppView::Keybinds => {
				self.open_section(Page::General, KEYBINDS_SECTION);
				Page::General
			}
			extensions::AppView::Storage => Page::Storage,
			extensions::AppView::Updates => Page::Updates,
			_ => return,
		};
		self.settings.query.clear();
		self.settings.open = true;
	}

	pub(super) fn theme_preview_navigation(&mut self, ui: &mut egui::Ui) {
		if self.extensions.begin_gallery_preview(ui.ctx()) {
			self.settings.open = false;
		}
		if self.settings.open {
			self.extensions.stop_theme_preview(ui.ctx());
		} else if self.extensions.theme_preview_bar(
			ui,
			if self.shows_title_bar() {
				design::TRAFFIC_LIGHT_INSET
			} else {
				0.0
			},
		) {
			self.settings.open = true;
			self.settings.page = Page::Themes;
			self.settings.query.clear();
		}
	}
	/// Opens settings on the last viewed page, as the macOS Settings menu item does.
	pub fn open_settings(&mut self) {
		self.settings.open = true;
	}
	pub fn open_update_settings(&mut self) {
		self.settings.open = true;
		self.settings.page = Page::Updates;
		self.settings.query.clear();
	}

	pub(super) fn keybinds_shortcut(&mut self, ctx: &egui::Context) {
		if !self.server_settings.is_open()
			&& !self.switcher.is_open()
			&& !self.ime_active
			&& !egui::Popup::is_any_open(ctx)
			&& ctx.memory(|memory| memory.top_modal_layer().is_none() || self.settings.open)
			&& ctx.input(|input| {
				input.focused
					&& !input
						.events
						.iter()
						.any(|event| matches!(event, egui::Event::Ime(_)))
			}) && ctx.input_mut(|input| {
			self.keybinds
				.chords(model::KeybindAction::ShowShortcuts)
				.any(|chord| crate::keybinds::pressed(input, chord))
		}) {
			self.settings.open = true;
			self.open_section(Page::General, KEYBINDS_SECTION);
			self.settings.query.clear();
		}
	}

	pub fn extension_settings_page(&self) -> Option<extensions::ExtensionKind> {
		if !self.settings.open {
			return None;
		}
		match self.settings.page {
			Page::Themes => Some(extensions::ExtensionKind::Theme),
			Page::Extensions => Some(extensions::ExtensionKind::Plugin),
			_ => None,
		}
	}

	pub fn voice_settings_open(&self) -> bool {
		self.settings.open && self.settings.page == Page::Voice
	}

	pub(super) fn open_notification_settings(&mut self) {
		self.settings.open = true;
		self.settings.page = Page::Notifications;
		self.settings.query.clear();
	}
	pub(super) fn open_voice_settings(&mut self) {
		self.settings.open = true;
		self.settings.page = Page::Voice;
		self.settings.query.clear();
	}

	/// Fixture-only entry point: opens the theme maker on the requested editor tab.
	#[cfg(feature = "demo")]
	pub fn preview_theme_maker(&mut self, tab: &str) {
		self.preview_settings("themes");
		self.extensions.preview_theme_maker(tab);
	}
	/// Fixture-only entry point for the native offline settings preview.
	pub fn preview_settings(&mut self, page: &str) {
		self.settings.open = true;
		// A label wins; keywords still reach pages that were merged or renamed ("activity").
		if let Some(page) = Page::ALL
			.into_iter()
			.find(|candidate| {
				candidate
					.label(Language::English)
					.to_lowercase()
					.contains(page)
			})
			.or_else(|| {
				Page::ALL
					.into_iter()
					.find(|candidate| candidate.matches(page, Language::English))
			}) {
			self.settings.page = page;
		}
	}
	pub(super) fn show_settings(
		&mut self,
		ctx: &egui::Context,
		state: &mut State,
		commands: &mut Vec<client_core::Command>,
	) {
		let colors = design::palette_for(ctx);
		let size = ctx.content_rect().size() - egui::vec2(32.0, 40.0);
		let width = size.x.clamp(280.0, 1100.0);
		let height = size.y.clamp(240.0, 820.0);
		let wide = width >= 620.0;
		let modal = egui::Modal::new(egui::Id::unique("user-settings"))
			.backdrop_color(egui::Color32::from_black_alpha(180))
			.frame(
				egui::Frame::new()
					.fill(colors.chat.to_opaque())
					.corner_radius(crate::dialog::RADIUS)
					.shadow(ctx.style_of(ctx.theme()).visuals.window_shadow)
					.stroke(egui::Stroke::new(1.0, colors.border)),
			)
			.show(ctx, |ui| {
				ui.set_width(width);
				ui.set_height(height);
				if wide {
					egui::Panel::left("settings-navigation")
						.exact_size(232.0)
						.resizable(false)
						.frame(
							egui::Frame::new()
								.fill(colors.sidebar.to_opaque())
								.corner_radius(egui::CornerRadius {
									nw: crate::dialog::RADIUS,
									sw: crate::dialog::RADIUS,
									ne: 0,
									se: 0,
								})
								.inner_margin(egui::Margin {
									left: 10,
									right: 8,
									top: 12,
									bottom: 12,
								}),
						)
						.show(ui, |ui| self.settings_navigation(ui, state));
				}
				egui::CentralPanel::default()
					.frame(egui::Frame::new().inner_margin(egui::Margin {
						left: if wide { 14 } else { 16 },
						right: 10,
						top: 8,
						bottom: 0,
					}))
					.show(ui, |ui| {
						let editing_theme =
							self.settings.page == Page::Themes && self.extensions.editing_theme();
						let heading = if editing_theme {
							self.language.text("theme-maker")
						} else {
							self.settings.page.row_label(self.language)
						};
						ui.horizontal(|ui| {
							ui.label(
								design::semibold(ui, &heading, 15.0).color(colors.text_strong),
							);
							ui.with_layout(
								egui::Layout::right_to_left(egui::Align::Center),
								|ui| {
									if close_button(ui).clicked() {
										self.settings.open = false;
									}
								},
							);
						});
						if !wide {
							ui.add_space(8.0);
							self.settings_search(ui);
							crate::design::Select::from_id_salt("settings-page")
								.selected_text(self.settings.page.label(self.language))
								.width(ui.available_width())
								.show_ui(ui, |ui| {
									for page in Page::ALL {
										if page.matches(
											&self.settings.query.to_lowercase(),
											self.language,
										) {
											ui.selectable_value(
												&mut self.settings.page,
												page,
												page.label(self.language),
											);
										}
									}
								});
						}
						ui.add_space(16.0);
						if self.settings.page == Page::Themes && self.extensions.editing_theme() {
							self.extensions.theme_editor_toolbar(ui);
							ui.add_space(12.0);
						}
						egui::ScrollArea::vertical()
							.id_salt((
								"settings-content",
								self.settings.page as u8,
								self.extensions.theme_editor_tab_key(),
							))
							.auto_shrink([false, false])
							.show(ui, |ui| {
								crate::dialog::page_fade(
									ui,
									egui::Id::unique((
										"settings-content",
										self.settings.page as u8,
									)),
								);
								let full = ui.available_width();
								let column = (full - 16.0).min(660.0);
								ui.horizontal_top(|ui| {
									ui.add_space(
										((full - column) / 2.0 - ui.spacing().item_spacing.x)
											.max(0.0),
									);
									ui.vertical(|ui| {
										ui.set_width(column);
										self.settings_page(ui, state, commands);
									});
								});
							});
					});
			});
		if modal.should_close() {
			self.settings.open = false;
		}
		if self.keybind_capture.is_none()
			&& ctx.input_mut(|input| {
				self.keybinds
					.chords(model::KeybindAction::CloseOverlay)
					.any(|chord| crate::keybinds::pressed_exact(input, chord))
			}) {
			self.settings.open = false;
		}
		if !self.settings.open {
			self.settings.messaging_permissions.requested = false;
		}
	}

	/// The selected page inside the centered content column.
	fn settings_page(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<client_core::Command>,
	) {
		let colors = design::palette(ui);
		ui.add_space(24.0);
		ui.spacing_mut().item_spacing.y = 12.0;
		let query = self.settings.query.to_lowercase();
		if !Page::ALL
			.into_iter()
			.any(|p| p.matches(&query, self.language))
		{
			ui.label(
				design::semibold(ui, self.language.text("no-settings-found"), 16.0)
					.color(colors.text_strong),
			);
			ui.weak(self.language.text("search-suggestion"));
			return;
		}
		if self.settings.sections.page != self.settings.page {
			self.settings.sections = SectionNav {
				page: self.settings.page,
				..SectionNav::default()
			};
		}
		match self.settings.page {
			Page::General => self.general_settings(ui, state.demo),
			Page::LanguageTime => self.language_time_settings(ui),
			Page::Account => self.account_page(ui, state),
			Page::Profile => self
				.settings
				.editor
				.show(ui, state, &mut self.avatars, commands),
			Page::Appearance => self.appearance_settings(ui, state.demo),
			Page::Accessibility => self.accessibility_settings(ui, state.demo),
			Page::MessagingPermissions => self.messaging_permissions_settings(ui, state, commands),
			Page::Notifications => self.notification_settings(ui, state.demo),
			Page::Activity => self.registered_games_settings(ui, state.demo),
			Page::ActivityPrivacy => self.activity_privacy_settings(ui, state),
			Page::Voice => self.voice_video_page(ui, state, commands),
			Page::Storage => self.storage_page(ui, state),
			Page::Updates => self.update_settings(ui, state.demo),
			Page::Developer => self.developer_settings(ui),
			Page::Nitro
			| Page::Boost
			| Page::Subscriptions
			| Page::Gifts
			| Page::Billing
			| Page::Connections => self.account_data_page(ui, state, commands, self.settings.page),
			Page::Extensions | Page::Themes => {
				self.extensions
					.select_themes(self.settings.page == Page::Themes);
				self.extensions.settings(ui, state);
			}
		}
		if state.demo {
			ui.add_space(20.0);
			design::hint(ui, &self.language.text("offline-preview"));
		}
		ui.add_space(24.0);
	}

	fn settings_navigation(&mut self, ui: &mut egui::Ui, state: &State) {
		let colors = design::palette(ui);
		let language = self.language;
		self.settings_profile_header(ui, state);
		ui.add_space(8.0);
		self.settings_search(ui);
		ui.add_space(8.0);
		egui::ScrollArea::vertical()
			.id_salt("settings-navigation-scroll")
			.auto_shrink([false, false])
			.show(ui, |ui| {
				ui.spacing_mut().item_spacing.y = 2.0;
				let query = self.settings.query.to_lowercase();
				let mut first = true;
				for (heading, pages) in Page::SECTIONS {
					let visible: Vec<Page> = pages
						.iter()
						.copied()
						.filter(|page| page.matches(&query, language))
						.collect();
					if visible.is_empty() {
						continue;
					}
					if !first {
						section_divider(ui);
					}
					first = false;
					if let Some(heading) = heading {
						ui.horizontal(|ui| {
							ui.add_space(6.0);
							ui.label(
								design::medium(ui, language.text(heading), 12.0)
									.color(colors.muted),
							);
						});
					}
					for page in visible {
						let open = self.settings.page == page;
						if sidebar_row(ui, page.icon(), &page.row_label(language), open, false)
							.clicked()
						{
							self.open_settings_page(page);
						}
						if !open {
							continue;
						}
						let sections = page.sections();
						if sections.len() > 1 {
							for (index, key) in sections.iter().enumerate() {
								let active = self.settings.sections.page == page
									&& self.settings.sections.active == index;
								if sidebar_sub_row(ui, &language.text(key), active).clicked() {
									self.open_section(page, index);
								}
							}
						}
						if page == Page::MessagingPermissions {
							for tab in crate::messaging_permissions::Tab::ALL {
								if sidebar_sub_row(
									ui,
									tab.label(),
									self.settings.messaging_permissions.active == tab,
								)
								.clicked()
								{
									self.settings.messaging_permissions.jump = Some(tab);
									self.settings.messaging_permissions.active = tab;
								}
							}
						}
						if page == Page::Notifications {
							for tab in crate::notification_settings::Tab::ALL {
								if sidebar_sub_row(
									ui,
									tab.label(),
									self.settings.notifications.active == tab,
								)
								.clicked()
								{
									self.settings.notifications.jump = Some(tab);
									self.settings.notifications.active = tab;
								}
							}
						}
					}
				}
				section_divider(ui);
				self.settings_logout(ui, state.demo);
				ui.add_space(12.0);
				ui.horizontal(|ui| {
					ui.add_space(6.0);
					ui.label(
						RichText::new(format!("Serein {}", self.build.version))
							.size(11.0)
							.color(colors.muted),
					);
				});
			});
	}

	fn open_section(&mut self, page: Page, section: usize) {
		self.open_settings_page(page);
		self.settings.sections = SectionNav {
			page,
			active: section,
			jump: Some(section),
		};
	}

	fn open_settings_page(&mut self, page: Page) {
		if page == Page::MessagingPermissions && self.settings.page != page {
			self.settings.messaging_permissions.requested = false;
		}
		self.settings.page = page;
	}

	/// Discord's sidebar header: avatar, name and an edit-profile line that opens Profile.
	fn settings_profile_header(&mut self, ui: &mut egui::Ui, state: &State) {
		let colors = design::palette(ui);
		let name = state.user.as_ref().map_or_else(
			|| self.language.text("page-account"),
			|user| user.name.clone(),
		);
		let edit = self.language.text("settings-account-page-edit-profile");
		let (rect, response) =
			ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), egui::Sense::click());
		response
			.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), &edit));
		if response.hovered() || response.has_focus() {
			ui.painter().rect_filled(rect, 8, colors.hover);
		}
		let avatar = egui::Rect::from_center_size(
			egui::pos2(rect.left() + 26.0, rect.center().y),
			egui::Vec2::splat(40.0),
		);
		let avatar_clicked = ui
			.scope_builder(egui::UiBuilder::new().max_rect(avatar), |ui| {
				match &state.user {
					Some(user) => self
						.avatars
						.show_plain(ui, user, 40.0, state.demo)
						.clicked(),
					None => {
						design::avatar(ui, &name, 40.0);
						false
					}
				}
			})
			.inner;
		let left = avatar.right() + 10.0;
		let width = rect.right() - left - 8.0;
		paint_truncated(
			ui,
			egui::pos2(left, rect.center().y - 9.0),
			&name,
			egui::FontId::new(15.0, design::semibold_family(ui.ctx())),
			colors.text_strong,
			width,
		);
		let (line, _) = paint_truncated(
			ui,
			egui::pos2(left, rect.center().y + 10.0),
			&edit,
			egui::FontId::new(12.0, design::medium_family(ui.ctx())),
			colors.muted,
			width - 18.0,
		);
		icons::paint(
			ui.painter(),
			icons::Icon::Pencil,
			egui::Rect::from_center_size(
				egui::pos2(line.right() + 10.0, line.center().y),
				egui::Vec2::splat(12.0),
			),
			colors.muted,
		);
		if response.clicked() || avatar_clicked {
			self.open_settings_page(Page::Profile);
		}
	}

	fn settings_search(&mut self, ui: &mut egui::Ui) {
		let colors = design::palette(ui);
		let language = self.language;
		egui::Frame::new()
			.fill(colors.raised)
			.corner_radius(6)
			.inner_margin(egui::Margin::symmetric(8, 4))
			.show(ui, |ui| {
				ui.horizontal(|ui| {
					ui.spacing_mut().item_spacing.x = 6.0;
					let response = ui.add(
						egui::TextEdit::singleline(&mut self.settings.query)
							.align(egui::Align2::LEFT_CENTER)
							.hint_text(language.text("search"))
							.char_limit(64)
							.frame(egui::Frame::NONE)
							.desired_width(ui.available_width() - 24.0),
					);
					icons::inline(ui, icons::Icon::Search, 16.0, colors.muted);
					if response.changed() {
						let query = self.settings.query.to_lowercase();
						if !self.settings.page.matches(&query, language)
							&& let Some(page) =
								Page::ALL.into_iter().find(|p| p.matches(&query, language))
						{
							self.settings.page = page;
						}
					}
				});
			});
	}

	fn settings_logout(&mut self, ui: &mut egui::Ui, demo: bool) {
		let label = self
			.language
			.text(if demo { "exit-preview" } else { "log-out" });
		if sidebar_row(ui, icons::Icon::SignOut, &label, false, true).clicked() {
			self.logout_requested = true;
			self.settings.open = false;
		}
	}

	fn account_page(&mut self, ui: &mut egui::Ui, state: &State) {
		let colors = design::palette(ui);
		let name = state
			.user
			.as_ref()
			.map_or("Your account", |u| u.name.as_str())
			.to_owned();
		egui::Frame::new()
			.fill(colors.raised)
			.stroke(egui::Stroke::new(1.0, colors.border))
			.corner_radius(8)
			.show(ui, |ui| {
				ui.set_width(ui.available_width());
				ui.spacing_mut().item_spacing.y = 0.0;
				let (banner, _) = ui.allocate_exact_size(
					egui::vec2(ui.available_width(), 96.0),
					egui::Sense::hover(),
				);
				ui.painter().rect_filled(
					banner,
					egui::CornerRadius {
						nw: 8,
						ne: 8,
						sw: 0,
						se: 0,
					},
					colors.accent,
				);
				egui::Frame::new()
					.inner_margin(egui::Margin {
						left: 16,
						right: 16,
						top: 12,
						bottom: 16,
					})
					.show(ui, |ui| {
						ui.set_width(ui.available_width());
						ui.horizontal(|ui| {
							ui.add_space(96.0);
							ui.vertical(|ui| {
								ui.spacing_mut().item_spacing.y = 2.0;
								ui.add(
									egui::Label::new(
										design::semibold(ui, name.clone(), 20.0)
											.color(colors.text_strong),
									)
									.truncate(),
								);
								ui.label(
									RichText::new(crate::i18n::translate_if_key(if state.demo {
										"settings-account-page-offline-preview-synthetic-account"
									} else {
										"settings-account-page-signed-in-with-your-discord-account"
									}))
									.size(13.0)
									.color(colors.muted),
								);
							});
						});
						ui.add_space(16.0);
						egui::Frame::new()
							.fill(colors.chat)
							.corner_radius(8)
							.inner_margin(egui::Margin::symmetric(16, 12))
							.show(ui, |ui| {
								ui.set_width(ui.available_width());
								ui.spacing_mut().item_spacing.y = 10.0;
								account_row(ui, "settings-account-page-display-name", &name);
								ui.separator();
								account_row(
									ui,
									"settings-account-page-email-password-and-security",
									"Managed in Discord",
								);
							});
						ui.add_space(12.0);
						ui.horizontal(|ui| {
							ui.with_layout(
								egui::Layout::right_to_left(egui::Align::Center),
								|ui| {
									if design::button(
										ui,
										&crate::i18n::translate(
											"settings-account-page-edit-profile",
										),
										design::ButtonKind::Outline,
									)
									.clicked()
									{
										self.settings.page = Page::Profile;
									}
								},
							);
						});
					});
				// Avatar overlapping the banner edge, ringed by the card surface.
				let avatar = egui::Rect::from_min_size(
					banner.left_bottom() + egui::vec2(16.0, -40.0),
					egui::Vec2::splat(80.0),
				);
				ui.painter()
					.circle_filled(avatar.center(), 44.0, colors.raised);
				ui.scope_builder(egui::UiBuilder::new().max_rect(avatar), |ui| {
					if let Some(user) = &state.user {
						self.avatars.with_avatar_animation(true, |avatars| {
							avatars.show(ui, user, 80.0, state.demo)
						});
					} else {
						design::avatar(ui, &name, 80.0);
					}
				});
			});
		let label = if state.demo {
			"Exit preview"
		} else {
			"Log out"
		};
		design::group(
			ui,
			&crate::i18n::translate("settings-account-page-session"),
			|ui| {
				if design::row(
					ui,
					label,
					Some(if state.demo {
						"settings-account-page-closes-the-offline-fixture-nothing-is-stored-for-the-preview"
					} else {
						"settings-account-page-removes-the-saved-login-and-clears-this-account-s-local"
					}),
					|ui| design::button(ui, label, design::ButtonKind::Danger),
				)
				.clicked()
				{
					self.logout_requested = true;
					self.settings.open = false;
				}
			},
		);
	}

	fn language_time_settings(&mut self, ui: &mut egui::Ui) {
		let language = self.language;
		let colors = design::palette(ui);
		ui.spacing_mut().item_spacing.y = 4.0;
		ui.label(
			design::semibold(ui, language.text("language-label"), 15.0).color(colors.text_strong),
		);
		ui.label(
			RichText::new(language.text("language-description"))
				.size(13.0)
				.color(colors.muted),
		);
		ui.add_space(4.0);
		crate::design::Select::from_id_salt("display-language")
			.selected_text(self.language.name(language))
			.width(ui.available_width())
			.show_ui(ui, |ui| {
				for candidate in Language::ALL {
					if language_option(ui, candidate, language, self.language == candidate)
						.clicked()
					{
						self.language = candidate;
					}
				}
			});
		ui.add_space(24.0);
		ui.label(
			design::semibold(ui, language.text("time-format-group"), 15.0)
				.color(colors.text_strong),
		);
		ui.add_space(4.0);
		for (format, key) in [
			(model::TimeFormat::Auto, "time-format-auto"),
			(model::TimeFormat::TwelveHour, "time-format-12-hour"),
			(model::TimeFormat::TwentyFourHour, "time-format-24-hour"),
		] {
			if design::radio_row(ui, self.time_format == format, &language.text(key), None)
				.clicked()
			{
				self.time_format = format;
			}
		}
		let format = (self.time_format, self.language);
		if self.settings.language_time_format != Some(format) {
			crate::local_time::set_format(self.time_format, self.language);
			self.settings.language_time_format = Some(format);
		}
	}

	/// Open at login and start minimised; shared by Settings > General and setup.
	pub(crate) fn startup_switches(&mut self, ui: &mut egui::Ui) {
		let language = self.language;
		ui.add_enabled_ui(self.startup_available && !self.startup_busy, |ui| {
			let label = language.text("general-open-at-startup");
			let description = language.text("general-open-at-startup-description");
			design::switch(ui, &label, Some(&description), &mut self.startup_enabled);
			design::card_divider(ui);
			ui.add_enabled_ui(self.startup_enabled, |ui| {
				let label = language.text("general-start-minimized");
				let description = language.text("general-start-minimized-description");
				design::switch(ui, &label, Some(&description), &mut self.startup_minimized);
			});
		});
		if !self.startup_available {
			design::hint(ui, &language.text("general-startup-unavailable"));
		} else if !self.startup_status.is_empty() {
			design::hint(ui, self.startup_status);
		}
	}

	/// Title bar and tray switches; shared by Settings > General and setup.
	pub(crate) fn window_switches(&mut self, ui: &mut egui::Ui) {
		let language = self.language;
		#[cfg(target_os = "linux")]
		{
			// Hiding every decoration and drawing Serein's own bar contradict each other:
			// whichever switch was just turned on wins. The "no title bar" switch is at the
			// bottom of this group, since it is only for tiling window managers.
			let label = language.text("general-custom-title-bar");
			let description = language.text("general-custom-title-bar-description");
			if design::switch(ui, &label, Some(&description), &mut self.custom_title_bar).changed()
				&& self.custom_title_bar
			{
				self.hide_window_decorations = false;
			}
			design::card_divider(ui);
		}
		#[cfg(any(target_os = "windows", target_os = "macos"))]
		{
			let label = language.text("general-hide-title-bar");
			let description = language.text("general-hide-title-bar-description");
			design::switch(ui, &label, Some(&description), &mut self.hide_title_bar);
			design::card_divider(ui);
		}
		ui.add_enabled_ui(self.tray_available, |ui| {
			let label = language.text(if cfg!(target_os = "macos") {
				"general-keep-menu-bar"
			} else {
				"general-keep-system-tray"
			});
			let description = language.text(if cfg!(target_os = "macos") {
				"general-menu-bar-description"
			} else if cfg!(target_os = "linux") {
				"general-linux-tray-description"
			} else {
				"general-windows-tray-description"
			});
			design::switch(ui, &label, Some(&description), &mut self.minimize_to_tray);
		});
		if !self.tray_available {
			design::hint(ui, &language.text("general-tray-unavailable"));
		} else if !self.tray_status.is_empty() {
			design::hint(ui, self.tray_status);
		}
		#[cfg(target_os = "linux")]
		{
			design::card_divider(ui);
			let label = language.text("general-hide-decorations");
			let description = language.text("general-hide-decorations-description");
			if design::switch(
				ui,
				&label,
				Some(&description),
				&mut self.hide_window_decorations,
			)
			.changed() && self.hide_window_decorations
			{
				self.custom_title_bar = false;
			}
		}
	}

	fn general_settings(&mut self, ui: &mut egui::Ui, _demo: bool) {
		let language = self.language;
		section_heading(
			ui,
			&mut self.settings.sections,
			0,
			&language.text("page-general"),
		);
		let title = language.text("general-startup");
		design::group(ui, &title, |ui| self.startup_switches(ui));
		let title = language.text("general-window");
		design::group(ui, &title, |ui| self.window_switches(ui));
		let title = language.text("settings-sub-custom-keybinds");
		section_heading(ui, &mut self.settings.sections, 1, &title);
		crate::keybinds::show_custom(ui, &mut self.keybinds, &mut self.keybind_capture);
		let title = language.text("settings-sub-default-keybinds");
		section_heading(ui, &mut self.settings.sections, 2, &title);
		crate::keybinds::show(
			ui,
			&mut self.keybinds,
			&mut self.keybind_capture,
			self.global_keybind_status,
		);
		let title = language.text("settings-sub-advanced");
		section_heading(ui, &mut self.settings.sections, 3, &title);
		let title = language.text("general-graphics");
		design::group(ui, &title, |ui| {
			let restart = language.text("general-gpu-restart");
			let detail = if self.gpu_adapter.is_empty() {
				restart
			} else {
				format!(
					"{} {}. {}",
					language.text("general-gpu-current-prefix"),
					self.gpu_adapter,
					restart
				)
			};
			let label = language.text("general-render-with");
			design::row(ui, &label, Some(&detail), |ui| {
				crate::design::Select::from_id_salt("gpu-preference")
					.selected_text(gpu_label(language, self.gpu_preference))
					.width(ui.available_width().min(220.0))
					.show_ui(ui, |ui| {
						for preference in model::GpuPreference::ALL {
							ui.selectable_value(
								&mut self.gpu_preference,
								preference,
								gpu_label(language, preference),
							)
							.on_hover_text(gpu_description(language, preference));
						}
					});
			});
			design::card_divider(ui);
			let label = language.text("general-graphics-api");
			let description = language.text(if self.graphics_api_env_override {
				"general-graphics-api-env-override"
			} else {
				"general-graphics-api-description"
			});
			design::row(ui, &label, Some(&description), |ui| {
				ui.add_enabled_ui(!self.graphics_api_env_override, |ui| {
					crate::design::Select::from_id_salt("graphics-api")
						.selected_text(graphics_api_label(language, self.graphics_api))
						.width(ui.available_width().min(220.0))
						.show_ui(ui, |ui| {
							for api in std::iter::once(model::GraphicsApi::Automatic).chain(
								self.available_graphics_apis
									.iter()
									.copied()
									.filter(|api| *api != model::GraphicsApi::Automatic),
							) {
								ui.selectable_value(
									&mut self.graphics_api,
									api,
									graphics_api_label(language, api),
								);
							}
						});
				});
			});
			design::card_divider(ui);
			let label = language.text("general-steady-refresh-rate");
			let description = language.text("general-steady-refresh-rate-description");
			design::switch(
				ui,
				&label,
				Some(&description),
				&mut self.steady_refresh_rate,
			);
		});
		design::card(ui, |ui| {
			design::row(
				ui,
				"setup-run-again",
				Some("setup-run-again-description"),
				|ui| {
					if design::button(ui, "setup-run-again-action", design::ButtonKind::Outline)
						.clicked()
					{
						self.preview_setup(0);
						self.settings.open = false;
					}
				},
			);
		});
	}

	/// Compact appearance popup for the signed-out header: mode, colour preset and zoom.
	/// Deliberately narrower than the settings page; everything else lives in Settings.
	pub fn appearance_menu(&mut self, ui: &mut egui::Ui) {
		let colors = design::palette(ui);
		ui.set_min_width(324.0);
		ui.set_max_width(324.0);
		ui.spacing_mut().item_spacing.y = 6.0;
		ui.label(design::eyebrow(
			ui,
			crate::i18n::translate("settings-appearance-menu-mode"),
			colors.muted,
		));
		theme_preference_cards(ui);
		ui.add_space(6.0);
		ui.label(design::eyebrow(
			ui,
			crate::i18n::translate("settings-appearance-menu-theme"),
			colors.muted,
		));
		let current = design::variant();
		ui.horizontal_wrapped(|ui| {
			ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);
			for variant in design::Variant::ALL {
				let swatch = design::builtin_colors(ui.visuals().dark_mode, variant);
				let selected = variant == current;
				if preset_swatch(ui, variant.label(), &swatch, selected).clicked() && !selected {
					design::set_variant(variant);
					design::apply(ui.ctx());
					self.theme_variant_changed = Some(variant);
				}
			}
		});
		ui.add_space(6.0);
		ui.label(design::eyebrow(
			ui,
			crate::i18n::translate("settings-appearance-menu-display"),
			colors.muted,
		));
		let mut value = self.reading_preferences;
		ui.spacing_mut().slider_width = 96.0;
		self.zoom_row(ui, &mut value);
		if value != self.reading_preferences {
			self.apply_reading_preferences(ui.ctx(), value);
		}
	}

	fn appearance_settings(&mut self, ui: &mut egui::Ui, demo: bool) {
		let language = self.language;
		section_heading(
			ui,
			&mut self.settings.sections,
			0,
			&language.text("settings-sub-theme"),
		);
		theme_preference_cards(ui);
		self.colour_preset_settings(ui);
		self.custom_font.show(ui);
		design::group(
			ui,
			&crate::i18n::translate("settings-appearance-settings-accent"),
			|ui| {
				self.primary_color_row(ui);
			},
		);
		design::group(
			ui,
			&crate::i18n::translate("settings-appearance-settings-window-effects"),
			|ui| {
				design::switch(
					ui,
					"settings-appearance-settings-transparency-blur",
					Some(
						"settings-appearance-settings-restart-serein-after-changing-this-themes-can-customize-effects-while",
					),
					&mut self.transparency_blur,
				);
				if self.transparency_blur {
					design::card_divider(ui);
					design::slider_row(
						ui,
						"settings-appearance-settings-transparency",
						None,
						&mut self.transparency,
						0..=100,
						"%",
					);
					ui.add_space(8.0);
					design::blur_control(
						ui,
						"settings-appearance-settings-blur",
						"settings-appearance-settings-the-system-applies-its-standard-blur-strength",
						&mut self.blur,
					);
				}
			},
		);
		self.cosmetic_settings(ui);
		self.layout_settings(ui, demo);
		section_heading(
			ui,
			&mut self.settings.sections,
			1,
			&language.text("settings-sub-messages"),
		);
		self.chat_reading_settings(ui, demo);
		section_heading(
			ui,
			&mut self.settings.sections,
			2,
			&language.text("settings-sub-chat-box"),
		);
		design::card(ui, |ui| {
			design::switch(
				ui,
				"settings-convert-emoticons",
				Some("settings-convert-emoticons-description"),
				&mut self.convert_emoticons,
			);
			design::card_divider(ui);
			let mut suggest = !self.hide_nitro_emojis;
			design::switch(
				ui,
				"settings-chat-settings-suggest-nitro-emojis",
				Some("settings-chat-settings-suggest-nitro-emojis-description"),
				&mut suggest,
			);
			self.hide_nitro_emojis = !suggest;
		});
		section_heading(
			ui,
			&mut self.settings.sections,
			3,
			&language.text("settings-sub-streamer-mode"),
		);
		self.streamer_mode_settings(ui);
		section_heading(
			ui,
			&mut self.settings.sections,
			4,
			&language.text("settings-sub-advanced"),
		);
		design::card(ui, |ui| {
			design::switch(
				ui,
				"settings-chat-settings-show-hidden-channels",
				Some("settings-chat-settings-show-channels-you-cannot-currently-access"),
				&mut self.show_hidden_channels,
			);
			design::card_divider(ui);
			design::switch(
				ui,
				"settings-preload-on-hover",
				Some("settings-preload-on-hover-description"),
				&mut self.preload_on_hover,
			);
		});
	}

	/// Discord's Accessibility page: visual density and reduced motion.
	fn accessibility_settings(&mut self, ui: &mut egui::Ui, demo: bool) {
		let language = self.language;
		let title = language.text("settings-sub-visual-density");
		section_heading(ui, &mut self.settings.sections, 0, &title);
		self.density_settings(ui, demo);
		let title = language.text("settings-sub-reduced-motion");
		section_heading(ui, &mut self.settings.sections, 1, &title);
		self.motion_settings(ui, demo);
	}

	/// Discord's Streamer Mode switches under Appearance.
	fn streamer_mode_settings(&mut self, ui: &mut egui::Ui) {
		let colors = design::palette(ui);
		let streamer = &mut self.streamer_mode;
		design::card(ui, |ui| {
			design::switch(
				ui,
				"streamer-mode-enable",
				Some("streamer-mode-enable-description"),
				&mut streamer.enabled,
			);
			design::card_divider(ui);
			design::switch(ui, "streamer-mode-automatic", None, &mut streamer.automatic);
			design::card_divider(ui);
			ui.label(
				design::semibold(
					ui,
					crate::i18n::translate("streamer-mode-when-enabled"),
					14.0,
				)
				.color(colors.text_strong),
			);
			for (key, value) in [
				("streamer-mode-hide-personal", &mut streamer.hide_personal),
				("streamer-mode-hide-invites", &mut streamer.hide_invites),
				("streamer-mode-disable-sounds", &mut streamer.disable_sounds),
				(
					"streamer-mode-disable-notifications",
					&mut streamer.disable_notifications,
				),
			] {
				design::card_divider(ui);
				design::switch(ui, key, None, value);
			}
			if cfg!(any(target_os = "windows", target_os = "macos")) {
				design::card_divider(ui);
				design::switch(
					ui,
					"streamer-mode-hide-window",
					Some("streamer-mode-hide-window-description"),
					&mut streamer.hide_window,
				);
			}
		});
	}

	/// Data & Privacy's voice encryption section.
	fn voice_encryption_settings(&mut self, ui: &mut egui::Ui, demo: bool) {
		design::card(ui, |ui| {
			ui.add_enabled_ui(!demo, |ui| {
				design::switch(
					ui,
					"settings-persistent-verification-codes",
					Some("settings-persistent-verification-codes-description"),
					&mut self.persistent_verification_codes,
				);
			});
			if !self.persistent_verification_status.is_empty() {
				design::hint(
					ui,
					&crate::i18n::translate_if_key(self.persistent_verification_status),
				);
			}
		});
	}

	/// Discord's Developer page.
	fn developer_settings(&mut self, ui: &mut egui::Ui) {
		design::card(ui, |ui| {
			design::switch(
				ui,
				"settings-developer-mode",
				Some("settings-developer-mode-description"),
				&mut self.developer_mode,
			);
		});
	}

	/// Primary colour picker with its reset; shared by Settings > Appearance and setup.
	pub(crate) fn primary_color_row(&mut self, ui: &mut egui::Ui) {
		let themed_accent = design::theme_sets_accent(ui.visuals().dark_mode);
		design::row(
			ui,
			"settings-appearance-settings-primary-color",
			Some(if themed_accent {
				"settings-appearance-settings-the-active-theme-brings-its-own-accent-it-takes-over"
			} else {
				"settings-appearance-settings-used-for-buttons-selection-and-message-highlights"
			}),
			|ui| {
				ui.add_enabled_ui(!themed_accent, |ui| {
					if self.primary_color.is_some()
						&& design::text_action(
							ui,
							&crate::i18n::translate("settings-appearance-settings-reset"),
						)
						.clicked()
					{
						self.primary_color = None;
					}
					let mut color = self.primary_color.unwrap_or(design::DEFAULT_PRIMARY_COLOR);
					if design::color_edit(ui, &mut color)
						.on_hover_text(crate::i18n::translate(
							"settings-appearance-settings-choose-primary-color",
						))
						.changed()
					{
						self.primary_color = Some(color);
					}
				});
			},
		);
	}

	/// Device-local switches for profile cosmetics. Hidden ones are neither drawn nor loaded.
	fn cosmetic_settings(&mut self, ui: &mut egui::Ui) {
		design::group(ui, &crate::i18n::translate("settings-cosmetics"), |ui| {
			cosmetic_switches(ui, &mut self.cosmetics, true);
			design::card_divider(ui);
			design::hint(
				ui,
				&crate::i18n::translate("settings-cosmetics-hidden-are-not-downloaded"),
			);
		});
	}

	/// Built-in presets plus enabled community themes, one swatch each.
	pub(crate) fn colour_preset_settings(&mut self, ui: &mut egui::Ui) {
		let current = design::variant();
		let mut presets: Vec<_> = design::Variant::ALL
			.into_iter()
			.map(|variant| {
				(
					Some(variant),
					None,
					variant.label().to_owned(),
					design::builtin_colors(ui.visuals().dark_mode, variant),
				)
			})
			.collect();
		presets.extend(self.extensions.entries.iter().filter_map(|entry| {
			if !entry.enabled || entry.manifest.kind != extensions::ExtensionKind::Theme {
				return None;
			}
			Some((
				None,
				Some(entry.manifest.id.clone()),
				entry.manifest.name.clone(),
				design::theme_preview_palette(ui, entry.theme_preview.as_ref()?),
			))
		}));
		design::group(
			ui,
			&crate::i18n::translate("settings-colour-preset-settings-colour-preset"),
			|ui| {
				ui.horizontal_wrapped(|ui| {
					ui.spacing_mut().item_spacing = egui::vec2(12.0, 10.0);
					for (variant, id, label, swatch) in presets {
						let selected = if let Some(active) = &self.extensions.active_theme {
							id.as_ref() == Some(active)
						} else {
							variant == Some(current)
						};
						let response = preset_swatch(ui, &label, &swatch, selected);
						if response.on_hover_text(&label).clicked()
							&& !selected && !self.extensions.busy
						{
							if let Some(variant) = variant {
								design::set_variant(variant);
								design::apply(ui.ctx());
								self.theme_variant_changed = Some(variant);
							}
							self.extensions
								.queue(ui.ctx(), crate::ExtensionRequest::SelectTheme { id });
						}
					}
				});
			},
		);
	}

	/// Discord's Activity Privacy page: whether the running game is shared.
	fn activity_privacy_settings(&mut self, ui: &mut egui::Ui, state: &State) {
		let title = self.language.text("settings-sub-shared-activity");
		section_heading(ui, &mut self.settings.sections, 0, &title);
		design::card(ui, |ui| {
			design::switch(
				ui,
				"settings-activity-settings-share-game-activity",
				Some(
					"settings-activity-settings-detect-running-games-and-ask-discord-to-share-them-as",
				),
				&mut self.share_game_activity,
			);
			design::card_divider(ui);
			let playing = self.own_game.is_some() || self.running_game.is_some();
			let action = if self.share_game_activity && state.gateway_connected && !state.demo {
				if self.discord_activity_sharing == Some(false) {
					Some(("settings-activity-enable-on-discord", true))
				} else if self.discord_activity_sharing_retry {
					Some(("settings-activity-check-again", false))
				} else {
					None
				}
			} else {
				None
			};
			// The game itself is shown under Current Game; this row is about sharing it.
			let title = if !self.share_game_activity {
				"settings-activity-sharing-is-off"
			} else if playing {
				"settings-activity-sharing-your-game"
			} else {
				"settings-activity-looking"
			};
			let detail = if state.demo {
				"settings-activity-demo-detail"
			} else {
				self.game_activity_status
			};
			design::row(ui, title, (!detail.is_empty()).then_some(detail), |ui| {
				if let Some((label, enable)) = action {
					ui.add_enabled_ui(!self.discord_activity_sharing_busy, |ui| {
						if design::button(ui, label, design::ButtonKind::Outline).clicked() {
							self.discord_activity_sharing_request = Some(enable);
						}
					});
				}
			});
		});
	}

	fn storage_page(&mut self, ui: &mut egui::Ui, state: &State) {
		let title = self.language.text("settings-sub-local-data");
		section_heading(ui, &mut self.settings.sections, 0, &title);
		design::group(
			ui,
			&crate::i18n::translate("settings-storage-page-local-storage"),
			|ui| {
				design::row(
					ui,
					"settings-storage-page-clear-cache",
					Some(
						"settings-storage-page-removes-cached-messages-and-media-drafts-and-your-login-stay",
					),
					|ui| {
						ui.add_enabled_ui(!state.demo, |ui| {
							if design::button(
								ui,
								&crate::i18n::translate("settings-storage-page-clear-cache"),
								design::ButtonKind::Outline,
							)
							.clicked()
							{
								self.clear_cache_requested = true;
							}
						});
					},
				);
				if !state.demo && !self.storage_status.is_empty() {
					design::hint(ui, self.storage_status);
				}
				design::card_divider(ui);
				design::hint(
					ui,
					&crate::i18n::translate(
						"settings-storage-page-messages-and-drafts-are-cached-on-this-device-inside-bounded",
					),
				);
			},
		);
		self.image_cache_settings(ui, state.demo);
		design::group(
			ui,
			&crate::i18n::translate("settings-storage-page-your-privacy"),
			|ui| {
				design::hint(
					ui,
					&crate::i18n::translate(
						"settings-storage-page-serein-does-not-collect-telemetry-or-upload-diagnostics-discord-retains",
					),
				);
			},
		);
		let title = self.language.text("settings-sub-voice-e2ee");
		section_heading(ui, &mut self.settings.sections, 1, &title);
		self.voice_encryption_settings(ui, state.demo);
	}
}

impl MessagingUi {
	/// Disk budgets for downloaded images, their measured use, and per-cache removal.
	fn image_cache_settings(&mut self, ui: &mut egui::Ui, demo: bool) {
		let usage = self.image_cache_usage;
		let detail = |description: &str, partition: model::CachePartition, limit: u32| {
			let used = match usage {
				Some(usage) => crate::i18n::translate_args(
					"settings-image-cache-usage",
					&[
						(
							"used",
							&crate::attachments::format_size(usage[partition as usize].bytes),
						),
						("limit", &cache_size_label(limit)),
					],
				),
				None => crate::i18n::translate("settings-image-cache-usage-unavailable"),
			};
			format!("{} {used}", crate::i18n::translate(description))
		};
		let media = detail(
			"settings-image-cache-media-description",
			model::CachePartition::Media,
			self.image_cache.media_mib,
		);
		let artwork = detail(
			"settings-image-cache-artwork-description",
			model::CachePartition::Artwork,
			self.image_cache.artwork_mib,
		);
		let mut clear = None;
		let cache = &mut self.image_cache;
		design::group(ui, &crate::i18n::translate("settings-image-cache"), |ui| {
			for (index, (title, detail, partition, value, presets)) in [
				(
					"settings-image-cache-media",
					&media,
					model::CachePartition::Media,
					&mut cache.media_mib,
					&model::MEDIA_CACHE_MIB[..],
				),
				(
					"settings-image-cache-artwork",
					&artwork,
					model::CachePartition::Artwork,
					&mut cache.artwork_mib,
					&model::ARTWORK_CACHE_MIB[..],
				),
			]
			.into_iter()
			.enumerate()
			{
				if index > 0 {
					design::card_divider(ui);
				}
				design::row(ui, title, Some(detail), |ui| {
					preset_combo(ui, title, value, presets, |mib| cache_size_label(*mib));
					ui.add_enabled_ui(!demo, |ui| {
						if design::text_action(
							ui,
							&crate::i18n::translate("settings-image-cache-clear"),
						)
						.on_hover_text(crate::i18n::translate(
							"settings-image-cache-clear-description",
						))
						.clicked()
						{
							clear = Some(partition);
						}
					});
				});
			}
			design::card_divider(ui);
			design::row(
				ui,
				"settings-image-cache-retention",
				Some("settings-image-cache-retention-description"),
				|ui| {
					preset_combo(
						ui,
						"settings-image-cache-retention",
						&mut cache.retention_days,
						&model::CACHE_RETENTION_DAYS,
						|days| {
							crate::i18n::translate_count(
								"settings-image-cache-days",
								usize::from(*days),
								&[],
							)
						},
					);
				},
			);
		});
		if clear.is_some() {
			self.clear_image_cache = clear;
		}
	}
}

/// Compact picker for one bounded preset list.
fn preset_combo<T: Copy + PartialEq>(
	ui: &mut egui::Ui,
	id: &str,
	value: &mut T,
	presets: &[T],
	label: impl Fn(&T) -> String,
) {
	crate::design::Select::from_id_salt(id)
		.selected_text(label(value))
		.width(ui.available_width().min(120.0))
		.show_ui(ui, |ui| {
			for preset in presets {
				ui.selectable_value(value, *preset, label(preset));
			}
		});
}

fn cache_size_label(mib: u32) -> String {
	if mib >= 1024 && mib.is_multiple_of(1024) {
		format!("{} GB", mib / 1024)
	} else {
		format!("{mib} MB")
	}
}

fn account_row(ui: &mut egui::Ui, label: &str, value: &str) {
	let label = crate::i18n::translate_if_key(label);
	let colors = design::palette(ui);
	ui.horizontal(|ui| {
		ui.vertical(|ui| {
			ui.set_width((ui.available_width() - 160.0).max(100.0));
			ui.spacing_mut().item_spacing.y = 2.0;
			ui.label(design::eyebrow(ui, label, colors.muted));
			ui.add(
				egui::Label::new(RichText::new(value).size(15.0).color(colors.text_strong))
					.truncate(),
			);
		});
	});
}

/// Sidebar entry in the settings modal; the selected page uses the strong surface and text.
pub(super) fn nav_item(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
	let label = crate::i18n::translate_if_key(label);
	let colors = design::palette(ui);
	let (rect, response) =
		ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click());
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, &label)
	});
	let hot = response.hovered() || response.has_focus();
	// Hover and selection ease in instead of snapping; idle rows cost no repaint.
	let time = ui.style().animation_time;
	let lit = ui
		.ctx()
		.animate_bool_with_time(response.id.with("nav-hover"), hot, time);
	let chosen =
		ui.ctx()
			.animate_bool_with_time(response.id.with("nav-selected"), selected, time * 1.5);
	if chosen > 0.0 || lit > 0.0 {
		let fill = colors
			.hover
			.gamma_multiply(lit.max(chosen))
			.lerp_to_gamma(colors.selected, chosen);
		ui.painter().rect_filled(rect, 8, fill);
	}
	if chosen > 0.0 {
		// Discord marks the open page with an accent rail at the left edge.
		let height = 16.0 * egui::emath::easing::cubic_out(chosen);
		ui.painter().rect_filled(
			egui::Rect::from_min_size(
				egui::pos2(rect.left(), rect.center().y - height * 0.5),
				egui::vec2(3.0, height),
			),
			2,
			colors.accent,
		);
	}
	if response.has_focus() {
		ui.painter().rect_stroke(
			rect.shrink(1.0),
			8,
			egui::Stroke::new(1.0, colors.accent),
			egui::StrokeKind::Inside,
		);
	}
	let color = if selected {
		colors.text_strong
	} else if hot {
		colors.text
	} else {
		colors.muted
	};
	// Long localized page names elide inside the row; hovering shows the full name.
	let mut job = egui::text::LayoutJob::simple_singleline(
		label.clone(),
		egui::FontId::new(15.0, design::medium_family(ui.ctx())),
		color,
	);
	job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 24.0);
	let galley = ui.painter().layout_job(job);
	let elided = galley.elided;
	ui.painter().galley(
		egui::pos2(rect.left() + 12.0, rect.center().y - galley.size().y / 2.0),
		galley,
		color,
	);
	if elided {
		response.on_hover_text(label)
	} else {
		response
	}
}

/// Paints one line from its left-center point, cut with an ellipsis past `width`. Returns the
/// painted rect and whether the text was cut.
fn paint_truncated(
	ui: &egui::Ui,
	left_center: egui::Pos2,
	text: &str,
	font: egui::FontId,
	color: egui::Color32,
	width: f32,
) -> (egui::Rect, bool) {
	let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
	job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(1.0));
	let galley = ui.painter().layout_job(job);
	let rect = egui::Rect::from_min_size(
		egui::pos2(left_center.x, left_center.y - galley.size().y / 2.0),
		galley.size(),
	);
	let elided = galley.elided;
	ui.painter().galley(rect.min, galley, color);
	(rect, elided)
}

/// Settings sidebar row with its icon; the open row stays filled.
/// Language list row: flag and native name, and the name in the current language on the right.
fn language_option(
	ui: &mut egui::Ui,
	candidate: Language,
	current: Language,
	selected: bool,
) -> egui::Response {
	let colors = design::palette(ui);
	let (rect, response) =
		ui.allocate_exact_size(egui::vec2(ui.available_width(), 36.0), egui::Sense::click());
	let native = candidate.name(current);
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, &native)
	});
	if selected || response.hovered() {
		let fill = if selected {
			colors.selected
		} else {
			colors.hover
		};
		ui.painter().rect_filled(rect, 4, fill);
	}
	let mut left = rect.left() + 10.0;
	if let Some(flag) = candidate
		.flag()
		.and_then(|flag| crate::emoji::image(ui.ctx(), flag, 20.0))
	{
		flag.paint_at(
			ui,
			egui::Rect::from_min_size(
				egui::pos2(left, rect.center().y - 10.0),
				egui::Vec2::splat(20.0),
			),
		);
		left += 30.0;
	}
	ui.painter().text(
		egui::pos2(left, rect.center().y),
		egui::Align2::LEFT_CENTER,
		native,
		egui::FontId::proportional(14.0),
		colors.text_strong,
	);
	if let Some(tag) = candidate.preference() {
		ui.painter().text(
			rect.right_center() - egui::vec2(10.0, 0.0),
			egui::Align2::RIGHT_CENTER,
			current.text(&format!("language-name-{tag}")),
			egui::FontId::proportional(13.0),
			colors.muted,
		);
	}
	response
}

fn sidebar_row(
	ui: &mut egui::Ui,
	icon: icons::Icon,
	label: &str,
	selected: bool,
	danger: bool,
) -> egui::Response {
	let colors = design::palette(ui);
	let (rect, response) =
		ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, label)
	});
	let hot = response.hovered() || response.has_focus();
	if selected || hot {
		let fill = if selected {
			colors.selected
		} else {
			colors.hover
		};
		ui.painter().rect_filled(rect, 6, fill);
	}
	if response.has_focus() {
		ui.painter().rect_stroke(
			rect.shrink(1.0),
			6,
			egui::Stroke::new(1.0, colors.accent),
			egui::StrokeKind::Inside,
		);
	}
	let ink = if danger {
		colors.danger
	} else if selected || hot {
		colors.text_strong
	} else {
		colors.text
	};
	icons::paint(
		ui.painter(),
		icon,
		egui::Rect::from_center_size(
			egui::pos2(rect.left() + 18.0, rect.center().y),
			egui::Vec2::splat(18.0),
		),
		ink,
	);
	let family = if selected {
		design::semibold_family(ui.ctx())
	} else {
		design::medium_family(ui.ctx())
	};
	let (_, elided) = paint_truncated(
		ui,
		egui::pos2(rect.left() + 36.0, rect.center().y),
		label,
		egui::FontId::new(15.0, family),
		ink,
		rect.width() - 44.0,
	);
	if elided {
		response.on_hover_text(label)
	} else {
		response
	}
}

/// Sub-item under an open sidebar row, on a rail below the row's icon.
fn sidebar_sub_row(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
	let label = crate::i18n::translate_if_key(label);
	let colors = design::palette(ui);
	let (rect, response) =
		ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, &label)
	});
	let hot = response.hovered() || response.has_focus();
	let rail = rect.left() + 18.0;
	let gap = ui.spacing().item_spacing.y;
	ui.painter().vline(
		rail,
		egui::Rangef::new(rect.top() - gap, rect.bottom()),
		egui::Stroke::new(1.0, colors.border),
	);
	if selected {
		ui.painter().vline(
			rail,
			egui::Rangef::new(rect.top() + 2.0, rect.bottom() - 2.0),
			egui::Stroke::new(2.0, colors.text_strong),
		);
	}
	if response.has_focus() {
		ui.painter().rect_stroke(
			rect.shrink(1.0),
			6,
			egui::Stroke::new(1.0, colors.accent),
			egui::StrokeKind::Inside,
		);
	}
	let ink = if selected || hot {
		colors.text_strong
	} else {
		colors.muted
	};
	let family = if selected {
		design::semibold_family(ui.ctx())
	} else {
		design::medium_family(ui.ctx())
	};
	let (_, elided) = paint_truncated(
		ui,
		egui::pos2(rect.left() + 36.0, rect.center().y),
		&label,
		egui::FontId::new(14.0, family),
		ink,
		rect.width() - 44.0,
	);
	if elided {
		response.on_hover_text(label)
	} else {
		response
	}
}

/// System's default keybinds section, the target of keybind shortcuts and deep links.
const KEYBINDS_SECTION: usize = 2;

/// Section title inside a settings page; keeps the sidebar sub-item in step with scrolling and
/// scrolls here when that sub-item is chosen.
pub(super) fn section_heading(ui: &mut egui::Ui, nav: &mut SectionNav, index: usize, title: &str) {
	let colors = design::palette(ui);
	if index > 0 {
		ui.add_space(16.0);
		let (rect, _) =
			ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
		ui.painter().hline(
			rect.x_range(),
			rect.center().y,
			egui::Stroke::new(1.0, colors.border),
		);
		ui.add_space(16.0);
	}
	let heading = ui.label(design::semibold(ui, title, 20.0).color(colors.text_strong));
	if heading.rect.top() <= ui.clip_rect().top() + 28.0 {
		nav.active = index;
	}
	if nav.jump == Some(index) {
		ui.scroll_to_rect(heading.rect.expand(8.0), Some(egui::Align::Min));
		nav.jump = None;
	}
}

/// Thin rule between settings sidebar groups.
fn section_divider(ui: &mut egui::Ui) {
	let colors = design::palette(ui);
	ui.add_space(6.0);
	let (rect, _) =
		ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
	ui.painter().hline(
		rect.shrink2(egui::vec2(6.0, 0.0)).x_range(),
		rect.center().y,
		egui::Stroke::new(1.0, colors.border),
	);
	ui.add_space(6.0);
}

/// Plain close button at the top right of the settings content.
pub(super) fn close_button(ui: &mut egui::Ui) -> egui::Response {
	let colors = design::palette(ui);
	let label = crate::i18n::translate("settings-close-control-close-settings-esc");
	let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(32.0), egui::Sense::click());
	response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), &label));
	let hot = response.hovered() || response.has_focus();
	if hot {
		ui.painter().rect_filled(rect, 6, colors.hover);
	}
	icons::paint(
		ui.painter(),
		icons::Icon::Close,
		egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(20.0)),
		if hot {
			colors.text_strong
		} else {
			colors.muted
		},
	);
	response.on_hover_text(label)
}

/// Dark, light or system cards with a miniature of each palette and a radio marker.
/// One colour-preset cell: the palette circles, the selection ring and the caption.
fn preset_swatch(
	ui: &mut egui::Ui,
	label: &str,
	swatch: &design::Palette,
	selected: bool,
) -> egui::Response {
	let label = crate::i18n::translate_if_key(label);
	let colors = design::palette(ui);
	let (rect, response) = ui.allocate_exact_size(egui::vec2(76.0, 70.0), egui::Sense::click());
	response.widget_info(|| {
		egui::WidgetInfo::selected(egui::Role::RadioButton, true, selected, &label)
	});
	let painter = &ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
	if response.hovered() || response.has_focus() {
		painter.rect_filled(rect, 6, colors.hover);
	}
	let center = egui::pos2(rect.center().x, rect.top() + 24.0);
	let [top, bottom] = swatch.backdrop.unwrap_or_else(|| {
		// Opaque presets run from their lightest surface to their darkest one.
		let mut surfaces = [swatch.chat, swatch.selected, swatch.base];
		surfaces.sort_by(|a, b| design::luminance(*b).total_cmp(&design::luminance(*a)));
		[surfaces[0], surfaces[2]]
	});
	gradient_circle(painter, center, 20.0, top, bottom);
	painter.circle_stroke(
		center,
		20.0,
		egui::Stroke::new(
			if selected { 2.5 } else { 1.0 },
			if selected {
				colors.accent
			} else {
				colors.border
			},
		),
	);
	if selected {
		painter.circle_filled(center, 10.0, colors.accent);
		icons::paint(
			painter,
			icons::Icon::Check,
			egui::Rect::from_center_size(center, egui::Vec2::splat(12.0)),
			colors.accent_text,
		);
	}
	painter.text(
		egui::pos2(rect.center().x, rect.bottom() - 10.0),
		egui::Align2::CENTER_CENTER,
		&label,
		egui::FontId::proportional(11.0),
		if selected {
			colors.text_strong
		} else {
			colors.muted
		},
	);
	response
}

/// Diagonal gradient from `top` (top-left) to `bottom` (bottom-right), like the window backdrop.
fn gradient_circle(
	painter: &egui::Painter,
	center: egui::Pos2,
	radius: f32,
	top: egui::Color32,
	bottom: egui::Color32,
) {
	const SEGMENTS: u32 = 48;
	let color_at = |offset: egui::Vec2| {
		let t = (offset.x + offset.y) / (2.0 * std::f32::consts::SQRT_2 * radius) + 0.5;
		top.lerp_to_gamma(bottom, t)
	};
	let mut mesh = egui::Mesh::default();
	mesh.colored_vertex(center, color_at(egui::Vec2::ZERO));
	for i in 0..SEGMENTS {
		let offset =
			egui::Vec2::angled(i as f32 * std::f32::consts::TAU / SEGMENTS as f32) * radius;
		mesh.colored_vertex(center + offset, color_at(offset));
		mesh.add_triangle(0, 1 + i, 1 + (i + 1) % SEGMENTS);
	}
	painter.add(egui::Shape::mesh(mesh));
}

/// One switch per cosmetic under a master that shows everything or nothing; Settings and
/// first-run setup share it; setup leaves out the per-cosmetic `details` to fit one screen.
pub(crate) fn cosmetic_switches(
	ui: &mut egui::Ui,
	cosmetics: &mut model::CosmeticPreferences,
	details: bool,
) {
	let mut all = *cosmetics == model::CosmeticPreferences::SHOWN;
	if design::switch(
		ui,
		"settings-cosmetics-all",
		Some("settings-cosmetics-all-description"),
		&mut all,
	)
	.changed()
	{
		*cosmetics = if all {
			model::CosmeticPreferences::SHOWN
		} else {
			model::CosmeticPreferences {
				solid_names: true,
				hide_clan_tags: true,
				..Default::default()
			}
		};
	}
	let mut gradients = !cosmetics.solid_names;
	let mut clan_tags = !cosmetics.hide_clan_tags;
	let switches = [
		(
			"settings-cosmetics-decorations",
			"settings-cosmetics-decorations-description",
			&mut cosmetics.decorations,
		),
		(
			"settings-cosmetics-nameplates",
			"settings-cosmetics-nameplates-description",
			&mut cosmetics.nameplates,
		),
		(
			"settings-cosmetics-effects",
			"settings-cosmetics-effects-description",
			&mut cosmetics.effects,
		),
		(
			"settings-cosmetics-frames",
			"settings-cosmetics-frames-description",
			&mut cosmetics.frames,
		),
		(
			"settings-cosmetics-name-styles",
			"settings-cosmetics-name-styles-description",
			&mut cosmetics.name_styles,
		),
		(
			"settings-cosmetics-gradient-names",
			"settings-cosmetics-gradient-names-description",
			&mut gradients,
		),
		(
			"settings-cosmetics-clan-tags",
			"settings-cosmetics-clan-tags-description",
			&mut clan_tags,
		),
	];
	if details {
		for (label, detail, value) in switches {
			design::card_divider(ui);
			design::switch(ui, label, Some(detail), value);
		}
	} else {
		// Labels alone fit two to a row; each row is its own pair so a wrapped label keeps
		// its neighbour level with it.
		design::card_divider(ui);
		ui.spacing_mut().item_spacing.x = 24.0;
		let mut switches = switches.into_iter();
		while let Some((label, _, value)) = switches.next() {
			let next = switches.next();
			ui.columns(2, |columns| {
				design::switch(&mut columns[0], label, None, value);
				if let Some((label, _, value)) = next {
					design::switch(&mut columns[1], label, None, value);
				}
			});
		}
	}
	cosmetics.solid_names = !gradients;
	cosmetics.hide_clan_tags = !clan_tags;
}

pub(crate) fn theme_preference_cards(ui: &mut egui::Ui) {
	let colors = design::palette(ui);
	let current = ui.ctx().options(|options| options.theme_preference);
	let mut chosen = None;
	ui.horizontal(|ui| {
		ui.spacing_mut().item_spacing.x = 10.0;
		let width = ((ui.available_width() - 20.0) / 3.0).clamp(88.0, 240.0);
		// Narrow cards (the signed-out appearance popup) cannot hold the long system label.
		for (preference, label) in [
			(
				egui::ThemePreference::Dark,
				"theme-editor-appearance-switch-dark",
			),
			(
				egui::ThemePreference::Light,
				"theme-editor-appearance-switch-light",
			),
			(
				egui::ThemePreference::System,
				if width < 140.0 {
					"language-system"
				} else {
					"settings-appearance-sync-with-system"
				},
			),
		] {
			let label = crate::i18n::translate_if_key(label);
			let selected = current == preference;
			let (rect, response) =
				ui.allocate_exact_size(egui::vec2(width, 76.0), egui::Sense::click());
			response.widget_info(|| {
				egui::WidgetInfo::selected(egui::Role::RadioButton, true, selected, &label)
			});
			let painter = ui.painter();
			painter.rect(
				rect,
				8,
				if response.hovered() {
					colors.hover
				} else {
					colors.raised
				},
				egui::Stroke::new(
					if selected { 2.0 } else { 1.0 },
					if selected {
						colors.accent
					} else {
						colors.border
					},
				),
				egui::StrokeKind::Inside,
			);
			let swatch = egui::Rect::from_min_size(
				rect.min + egui::vec2(12.0, 12.0),
				egui::vec2(52.0, 34.0),
			);
			let variant = design::variant();
			let (left, right) = match preference {
				egui::ThemePreference::Dark => {
					let p = design::control_colors(true, variant);
					(p.sidebar.to_opaque(), p.chat.to_opaque())
				}
				egui::ThemePreference::Light => {
					let p = design::control_colors(false, variant);
					(p.sidebar.to_opaque(), p.chat.to_opaque())
				}
				egui::ThemePreference::System => (
					design::control_colors(true, variant).chat,
					design::control_colors(false, variant).chat,
				),
			};
			painter.rect_filled(swatch, 6, right);
			painter.rect_filled(
				swatch.with_max_x(swatch.left() + swatch.width() * 0.42),
				egui::CornerRadius {
					nw: 6,
					sw: 6,
					ne: 0,
					se: 0,
				},
				left,
			);
			painter.rect_stroke(
				swatch,
				6,
				egui::Stroke::new(1.0, colors.border),
				egui::StrokeKind::Inside,
			);
			let radio = egui::pos2(rect.right() - 20.0, rect.top() + 20.0);
			painter.circle_stroke(
				radio,
				8.0,
				egui::Stroke::new(
					2.0,
					if selected {
						colors.accent
					} else {
						colors.muted
					},
				),
			);
			if selected {
				painter.circle_filled(radio, 4.5, colors.accent);
			}
			painter.text(
				egui::pos2(rect.left() + 12.0, rect.bottom() - 14.0),
				egui::Align2::LEFT_CENTER,
				&label,
				egui::FontId::new(14.0, design::medium_family(ui.ctx())),
				if selected {
					colors.text_strong
				} else {
					colors.text
				},
			);
			if response.clicked() {
				chosen = Some(preference);
			}
		}
	});
	if let Some(preference) = chosen {
		ui.ctx().set_theme(preference);
	}
}

#[cfg(test)]
mod keybind_tests {
	use super::*;

	#[test]
	fn keybinds_shortcut_opens_page_and_respects_ime() {
		let ctx = egui::Context::default();
		let mut view = MessagingUi::default();
		for ime in [true, false] {
			view.ime_active = ime;
			view.settings.query = "theme".into();
			let mut output = ctx.run_ui(
				egui::RawInput {
					events: vec![egui::Event::Key {
						key: egui::Key::Slash,
						physical_key: None,
						pressed: true,
						repeat: false,
						modifiers: egui::Modifiers::COMMAND,
					}],
					..Default::default()
				},
				|ui| view.keybinds_shortcut(ui.ctx()),
			);
			output.textures_delta.clear();
			assert_eq!(view.settings.open, !ime);
			if !ime {
				assert!(view.settings.page == Page::General);
				assert_eq!(view.settings.sections.jump, Some(KEYBINDS_SECTION));
				assert!(view.settings.query.is_empty());
			}
		}
	}
}
