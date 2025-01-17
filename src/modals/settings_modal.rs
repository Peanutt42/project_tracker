use crate::components::{
	code_editor_dropdown_button, export_as_json_database_button, horizontal_seperator_padded,
	import_json_database_button, loading_screen, synchronization_settings_button,
	vertical_scrollable, vertical_scrollable_no_padding, LARGE_LOADING_SPINNER_SIZE,
};
use crate::icons::{icon_to_text, Bootstrap};
use crate::integrations::CodeEditor;
use crate::project_tracker::{Message, ProjectTrackerApp};
use crate::styles::{
	command_background_container_style, grey_text_style, link_color, logs_scrollable_style,
	markdown_background_container_style, rounded_container_style, text_input_style_default,
	tooltip_container_style, GAP, HEADING_TEXT_SIZE, LARGE_TEXT_SIZE, MONOSPACE_FONT,
	SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_SPACING_AMOUNT, SMALL_TEXT_SIZE,
	SPACING_AMOUNT,
};
use crate::synchronization::{
	browse_filesystem_synchronization_filepath_dialog, BaseSynchronization,
};
use crate::synchronization::{ServerConfig, ServerSynchronization, Synchronization};
use crate::{
	components::{
		dangerous_button, export_database_button, file_location, import_database_button,
		import_google_tasks_button, settings_tab_button, vertical_seperator,
		HORIZONTAL_SCROLLABLE_PADDING, ICON_FONT_SIZE,
	},
	integrations::import_google_tasks_dialog,
	modals::ErrorMsgModalMessage,
	styles::{card_style, PADDING_AMOUNT},
	DateFormatting, PreferenceAction, PreferenceMessage, Preferences,
};
use iced::alignment::Vertical;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::text::Span;
use iced::widget::{rich_text, scrollable, text_input, tooltip};
use iced::{
	alignment::Horizontal,
	keyboard,
	widget::{column, container, row, text, Column, Space},
	Alignment, Element,
	Length::Fill,
	Padding, Subscription, Task,
};
use iced::{time, Color, Length};
use iced_aw::card;
use project_tracker_core::DatabaseMessage;
use project_tracker_server::AdminInfos;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SettingsModalMessage {
	Open,
	Close,
	OpenTab(SettingTab),

	ShowPassword,
	HidePassword,

	ToggleCodeEditorDropdownExpanded,
	CollapseCodeEditorDropdown,
	SetCodeEditor(Option<CodeEditor>),

	SetDateFormatting(DateFormatting),

	ImportGoogleTasksFileDialog,
	ImportGoogleTasksFileDialogCanceled,

	BrowseSynchronizationFilepath,
	BrowseSynchronizationFilepathCanceled,

	SetServerHostname(String),
	SetServerPort(usize),
	SetServerPassword(String),
	InvalidPortInput,
}

impl From<SettingsModalMessage> for Message {
	fn from(value: SettingsModalMessage) -> Self {
		Message::SettingsModalMessage(value)
	}
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum SettingTab {
	#[default]
	General,
	Database,
	Shortcuts,
	CodeEditor,
	AdminInfos,
	About,
}

impl SettingTab {
	const ALL: [SettingTab; 6] = [
		SettingTab::General,
		SettingTab::Database,
		SettingTab::Shortcuts,
		SettingTab::CodeEditor,
		SettingTab::AdminInfos,
		SettingTab::About,
	];

	pub fn icon(&self) -> Bootstrap {
		match self {
			Self::General => Bootstrap::GearFill,
			Self::Database => Bootstrap::DatabaseFillGear,
			Self::Shortcuts => Bootstrap::Command,
			Self::CodeEditor => Bootstrap::CodeSlash,
			Self::AdminInfos => Bootstrap::BarChartFill,
			Self::About => Bootstrap::InfoSquare,
		}
	}

	pub fn label(&self) -> &'static str {
		match self {
			Self::General => "General",
			Self::Database => "Database",
			Self::Shortcuts => "Shortcuts",
			Self::CodeEditor => "Code Editor",
			Self::AdminInfos => "Admin Infos",
			Self::About => "About",
		}
	}

	fn view<'a>(
		&'a self,
		app: &'a ProjectTrackerApp,
		preferences: &'a Preferences,
		show_password: bool,
		code_editor_dropdown_expanded: bool,
	) -> Element<'a, Message> {
		match self {
			SettingTab::General => vertical_scrollable(preferences.view(&app.flags)).into(),
			SettingTab::Database => {
				vertical_scrollable(database_settings_tab_view(app, preferences, show_password))
					.into()
			}
			SettingTab::Shortcuts => vertical_scrollable(shortcuts_settings_tab_view()).into(),
			SettingTab::CodeEditor => vertical_scrollable(code_editor_settings_tab_view(
				preferences,
				code_editor_dropdown_expanded,
			))
			.into(),
			// no 'vertical_scrollable': loading_screen has 'Fill' height
			// --> 'admin_infos_settings_tab_view' calls 'vertical_scrollable' internally
			SettingTab::AdminInfos => admin_infos_settings_tab_view(
				app.synchronization
					.as_ref()
					.and_then(Synchronization::latest_admin_infos),
			),
			SettingTab::About => vertical_scrollable(about_settings_tab_view(app)).into(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub enum SettingsModal {
	Opened {
		selected_tab: SettingTab,
		show_password: bool,
		code_editor_dropdown_expanded: bool,
	},
	#[default]
	Closed,
}

impl SettingsModal {
	pub fn is_open(&self) -> bool {
		matches!(self, SettingsModal::Opened { .. })
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let listen_open_shortcut_subscription =
			keyboard::on_key_press(|key, modifiers| match key.as_ref() {
				keyboard::Key::Character(",") if modifiers.command() => {
					Some(SettingsModalMessage::Open.into())
				}
				_ => None,
			});

		let mut subscriptions = vec![listen_open_shortcut_subscription];

		if matches!(
			self,
			SettingsModal::Opened {
				selected_tab: SettingTab::AdminInfos,
				..
			}
		) {
			subscriptions
				.push(time::every(Duration::from_secs(2)).map(|_| Message::RequestAdminInfos));
		}

		Subscription::batch(subscriptions)
	}

	pub fn update(
		&mut self,
		message: SettingsModalMessage,
		preferences: &mut Option<Preferences>,
	) -> PreferenceAction {
		match message {
			SettingsModalMessage::Open => {
				*self = SettingsModal::Opened {
					selected_tab: SettingTab::default(),
					show_password: false,
					code_editor_dropdown_expanded: false,
				};
				PreferenceAction::None
			}
			SettingsModalMessage::Close => {
				*self = SettingsModal::Closed;
				PreferenceAction::None
			}
			SettingsModalMessage::OpenTab(tab) => {
				if let SettingsModal::Opened {
					selected_tab,
					show_password,
					code_editor_dropdown_expanded,
					..
				} = self
				{
					*selected_tab = tab;
					*show_password = false;
					*code_editor_dropdown_expanded = false;
				}
				match tab {
					SettingTab::AdminInfos => PreferenceAction::RequestAdminInfos,
					_ => PreferenceAction::None,
				}
			}

			SettingsModalMessage::ShowPassword => {
				if let SettingsModal::Opened { show_password, .. } = self {
					*show_password = true;
				}
				PreferenceAction::None
			}
			SettingsModalMessage::HidePassword => {
				if let SettingsModal::Opened { show_password, .. } = self {
					*show_password = false;
				}
				PreferenceAction::None
			}

			SettingsModalMessage::ToggleCodeEditorDropdownExpanded => {
				if let SettingsModal::Opened {
					code_editor_dropdown_expanded,
					..
				} = self
				{
					*code_editor_dropdown_expanded = !*code_editor_dropdown_expanded;
				}
				PreferenceAction::None
			}
			SettingsModalMessage::CollapseCodeEditorDropdown => {
				if let SettingsModal::Opened {
					code_editor_dropdown_expanded,
					..
				} = self
				{
					*code_editor_dropdown_expanded = false;
				}
				PreferenceAction::None
			}
			SettingsModalMessage::SetCodeEditor(code_editor) => {
				if let Some(preferences) = preferences {
					preferences.set_code_editor(code_editor);
				}
				if let SettingsModal::Opened {
					code_editor_dropdown_expanded,
					..
				} = self
				{
					*code_editor_dropdown_expanded = false;
				}
				PreferenceAction::None
			}

			SettingsModalMessage::ImportGoogleTasksFileDialog => {
				Task::perform(import_google_tasks_dialog(), move |result| match result {
					Some(result) => match result {
						Ok(projects) => DatabaseMessage::ImportProjects(projects).into(),
						Err(import_error) => ErrorMsgModalMessage::open_error(import_error),
					},
					None => SettingsModalMessage::ImportGoogleTasksFileDialogCanceled.into(),
				})
				.into()
			}
			SettingsModalMessage::ImportGoogleTasksFileDialogCanceled => PreferenceAction::None,

			SettingsModalMessage::BrowseSynchronizationFilepath => Task::perform(
				browse_filesystem_synchronization_filepath_dialog(),
				|file_synchronization| match file_synchronization {
					Some(file_synchronization) => {
						PreferenceMessage::SetSynchronization(Some(file_synchronization.into()))
							.into()
					}
					None => SettingsModalMessage::BrowseSynchronizationFilepathCanceled.into(),
				},
			)
			.into(),
			SettingsModalMessage::BrowseSynchronizationFilepathCanceled => PreferenceAction::None,

			SettingsModalMessage::SetDateFormatting(date_formatting) => match preferences {
				Some(preferences) => {
					preferences.update(PreferenceMessage::SetDateFormatting(date_formatting))
				}
				None => PreferenceAction::None,
			},

			SettingsModalMessage::SetServerHostname(new_hostname) => {
				if let Some(preferences) = preferences {
					if let Some(Synchronization::ServerSynchronization(server_synchronization)) =
						preferences.synchronization()
					{
						return PreferenceMessage::SetSynchronization(Some(
							Synchronization::ServerSynchronization(ServerSynchronization::new(
								ServerConfig {
									hostname: new_hostname,
									..server_synchronization.config.clone()
								},
							)),
						))
						.into();
					}
				}
				PreferenceAction::None
			}
			SettingsModalMessage::SetServerPort(new_port) => {
				if let Some(preferences) = preferences {
					if let Some(Synchronization::ServerSynchronization(server_synchronization)) =
						preferences.synchronization()
					{
						return PreferenceMessage::SetSynchronization(Some(
							Synchronization::ServerSynchronization(ServerSynchronization::new(
								ServerConfig {
									port: new_port,
									..server_synchronization.config.clone()
								},
							)),
						))
						.into();
					}
				}
				PreferenceAction::None
			}
			SettingsModalMessage::SetServerPassword(new_password) => {
				if let Some(preferences) = preferences {
					if let Some(Synchronization::ServerSynchronization(server_synchronization)) =
						preferences.synchronization()
					{
						return PreferenceMessage::SetSynchronization(Some(
							Synchronization::ServerSynchronization(ServerSynchronization::new(
								ServerConfig {
									password: new_password,
									..server_synchronization.config.clone()
								},
							)),
						))
						.into();
					}
				}
				PreferenceAction::None
			}
			SettingsModalMessage::InvalidPortInput => PreferenceAction::None,
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<Element<'a, Message>> {
		match self {
			SettingsModal::Closed => None,
			SettingsModal::Opened {
				selected_tab,
				show_password,
				code_editor_dropdown_expanded,
			} => app.preferences.as_ref().map(|preferences| {
				let tabs: Vec<Element<Message>> = SettingTab::ALL
					.iter()
					.map(|tab| settings_tab_button(*tab, *selected_tab).into())
					.collect();

				card(
					text("Settings").size(HEADING_TEXT_SIZE),
					row![
						vertical_scrollable_no_padding(
							Column::with_children(tabs)
								.width(150.0)
								.spacing(SMALL_SPACING_AMOUNT)
						),
						vertical_seperator(),
						selected_tab.view(
							app,
							preferences,
							*show_password,
							*code_editor_dropdown_expanded
						)
					]
					.spacing(SPACING_AMOUNT),
				)
				.max_width(900.0)
				.max_height(550.0)
				.close_size(LARGE_TEXT_SIZE)
				.on_close(SettingsModalMessage::Close.into())
				.style(card_style)
				.into()
			}),
		}
	}
}

fn database_settings_tab_view<'a>(
	app: &'a ProjectTrackerApp,
	preferences: &'a Preferences,
	show_password: bool,
) -> Element<'a, Message> {
	let synchronization = preferences.synchronization();
	let filesystem_synchronization_enabled = synchronization
		.as_ref()
		.map(Synchronization::is_filesystem)
		.unwrap_or(false);
	let server_synchronization_enabled = synchronization
		.as_ref()
		.map(Synchronization::is_server)
		.unwrap_or(false);

	column![
		row![
			container("Database file location: ").padding(HORIZONTAL_SCROLLABLE_PADDING),
			container(match app.flags.get_database_filepath() {
				Some(filepath) => file_location(filepath),
				None => text("could not find database filepath").into(),
			})
			.width(Fill)
			.align_x(Horizontal::Right),
		]
		.align_y(Alignment::Center),

		container(
			column![
				row![
					dangerous_button(
						Bootstrap::Trash,
						"Clear",
						Some("Clear Database".to_string()),
						DatabaseMessage::Clear
					),
					import_database_button(app.importing_database),
					export_database_button(app.exporting_database),
				]
				.spacing(SPACING_AMOUNT),

				row![
					import_json_database_button(app.importing_database),
					export_as_json_database_button(app.exporting_database),
				]
				.spacing(SPACING_AMOUNT),
			]
			.spacing(SPACING_AMOUNT)
		)
		.width(Fill)
		.align_x(Horizontal::Right),

		horizontal_seperator_padded(),

		column![
			row![
				text("Synchronization:"),

				container(
					row![
						synchronization_settings_button(
							"Off",
							synchronization.is_none(),
							PreferenceMessage::SetSynchronization(None).into(),
							true,
							false
						),
						synchronization_settings_button(
							"Filesystem",
							filesystem_synchronization_enabled,
							SettingsModalMessage::BrowseSynchronizationFilepath.into(),
							false,
							false
						),
						synchronization_settings_button(
							"Server",
							server_synchronization_enabled,
							PreferenceMessage::SetSynchronization(Some(ServerSynchronization::default().into())).into(),
							false,
							true
						)
					]
				)
				.width(Fill)
				.align_x(Horizontal::Right)
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center)
		]
		.push_maybe(synchronization.as_ref().map(|synchronization| synchronization.view(show_password)))
		.spacing(SPACING_AMOUNT),

		horizontal_seperator_padded(),

		column![
			row![
				"Import Google Tasks:",

				container(
					import_google_tasks_button()
				)
				.width(Fill)
				.align_x(Horizontal::Right),
			]
			.spacing(SPACING_AMOUNT)
			.align_y(Alignment::Center),

			container(
				rich_text![
					Span::new("Go to "),
					Span::new("https://myaccount.google.com/dashboard")
						.color(link_color(app.is_theme_dark()))
						.link(Message::OpenUrl("https://myaccount.google.com/dashboard".to_string())),
					Span::new(" and download the Tasks data.\nThen extract the Takeout.zip and import the \"Tasks.json\" file inside under the \"Tasks\" folder.")
				]
				.style(grey_text_style)
			)
			.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
		]
		.spacing(SPACING_AMOUNT),
	]
	.spacing(SPACING_AMOUNT)
	.into()
}

fn shortcuts_settings_tab_view() -> Element<'static, Message> {
	let shortcut = |name, shortcut| {
		row![
			text(name),
			Space::new(Fill, 0.0),
			container(container(shortcut).padding(SMALL_HORIZONTAL_PADDING))
				.style(rounded_container_style)
		]
		.spacing(SMALL_SPACING_AMOUNT)
	};

	column![
		shortcut("Open Settings:", "Ctrl + ,"),
		shortcut("Open Overview:", "Ctrl + H"),
		shortcut("New Project:", "Ctrl + Shift + N"),
		shortcut("Search Tasks:", "Ctrl + F"),
		shortcut("Delete Project:", "Ctrl + Del"),
		shortcut("Switch to lower Project:", "Ctrl + Tab"),
		shortcut("Switch to upper Project:", "Ctrl + Shift + Tab"),
		shortcut("New Task:", "Ctrl + N"),
		shortcut("Toggle Sidebar:", "Ctrl + B"),
		shortcut("Start/Pause/Resume Stopwatch:", "Space"),
		shortcut("Stop Stopwatch:", "Esc"),
	]
	.spacing(SPACING_AMOUNT)
	.into()
}

fn code_editor_settings_tab_view(
	preferences: &Preferences,
	dropdown_expanded: bool,
) -> Element<Message> {
	column![item(
		"Code Editor:",
		code_editor_dropdown_button(preferences.code_editor().as_ref(), dropdown_expanded),
	)]
	.push_maybe(
		match preferences.code_editor().as_ref()  {
			Some(CodeEditor::Custom { name, command }) => {
				Some(
					column![
						horizontal_seperator_padded(),
						item(
							"Custom Editor Name:",
							text_input("Custom Editor name", name.as_str())
								.width(Length::Fixed(350.0))
								.on_input(|new_name| SettingsModalMessage::SetCodeEditor(Some(
									CodeEditor::Custom {
										name: new_name,
										command: command.clone(),
									}
								))
								.into())
								.style(text_input_style_default)
						),
						item(
							row![
								"Custom Editor Command:",
								tooltip(
									icon_to_text(Bootstrap::QuestionCircleFill).size(ICON_FONT_SIZE),
									text("the file location formatted as \"filepath:line:column\" is added to the end of this command")
										.size(SMALL_TEXT_SIZE),
									tooltip::Position::Bottom,
								)
								.gap(GAP)
								.style(tooltip_container_style),
							]
							.spacing(SPACING_AMOUNT),
							text_input("custom_editor --open", command.as_str())
								.width(Length::Fixed(350.0))
								.on_input(|new_command| SettingsModalMessage::SetCodeEditor(Some(
									CodeEditor::Custom {
										name: name.clone(),
										command: new_command,
									}
								))
								.into())
								.style(text_input_style_default)
						),
						row![
							text("example command:")
								.width(Fill)
								.style(grey_text_style),

							container(
								rich_text![
									Span::new(command).color(Color::from_rgb8(154, 196, 248)),
									Span::new(" \"file.txt:42:15\"").color(Color::from_rgb8(162, 250, 163))
								]
							)
							.padding(Padding::new(SMALL_PADDING_AMOUNT))
							.style(command_background_container_style),
						]
						.align_y(Vertical::Center)
						.spacing(SPACING_AMOUNT)
					]
					.spacing(SPACING_AMOUNT),
				)
			},
			_ => None,
		},
	)
	.width(Fill)
	.spacing(SPACING_AMOUNT)
	.into()
}

fn admin_infos_settings_tab_view(admin_infos: Option<&AdminInfos>) -> Element<Message> {
	match admin_infos {
		Some(admin_infos) => vertical_scrollable(
			column![
				item(
					"Cpu Usage:",
					text(format!("{}%", (admin_infos.cpu_usage * 100.0).round()))
				),
				horizontal_seperator_padded(),
				item(
					"Cpu Temp:",
					text(
						admin_infos
							.cpu_temp
							.map(|cpu_temp| format!("{} °C", cpu_temp.round()))
							.unwrap_or("failed to get cpu_temp".to_string())
					)
				),
				horizontal_seperator_padded(),
				item("Ram Usage:", text(&admin_infos.ram_info),),
				horizontal_seperator_padded(),
				item("Uptime:", text(&admin_infos.uptime)),
				horizontal_seperator_padded(),
				item(
					"Native GUI Clients:",
					Column::with_children(
						admin_infos
							.connected_native_gui_clients
							.iter()
							.map(|connection| text(format!("{connection}")).into())
					)
				),
				horizontal_seperator_padded(),
				item(
					"Web Clients:",
					Column::with_children(
						admin_infos
							.connected_web_clients
							.iter()
							.map(|connection| text(format!("{connection}")).into())
					)
				),
				horizontal_seperator_padded(),
				text("Latest Logs:"),
				container(
					scrollable(
						container(text(&admin_infos.latest_logs_of_the_day).font(MONOSPACE_FONT))
							.style(markdown_background_container_style)
							.padding(Padding::new(PADDING_AMOUNT))
					)
					.height(Length::Fixed(350.0))
					.direction(Direction::Both {
						horizontal: Scrollbar::default(),
						vertical: Scrollbar::default(),
					})
					.anchor_bottom()
					.style(logs_scrollable_style)
				)
				.padding(Padding::default().right(PADDING_AMOUNT))
			]
			.spacing(SPACING_AMOUNT),
		)
		.into(),
		None => container(loading_screen(LARGE_LOADING_SPINNER_SIZE))
			.center(Fill)
			.into(),
	}
}

fn about_settings_tab_view(app: &ProjectTrackerApp) -> Element<Message> {
	let repository = env!("CARGO_PKG_REPOSITORY");

	let author_link = "https://github.com/Peanutt42";

	column![
		item(
			"Project Tracker:",
			text("Project Todo Tracker for personal programming projects")
		),
		item(
			"Author:",
			rich_text![Span::new(author_link)
				.color(link_color(app.is_theme_dark()))
				.link(Message::OpenUrl(author_link.to_string()))]
		),
		item("Version:", text(env!("CARGO_PKG_VERSION"))),
		item(
			"Repository:",
			rich_text![Span::new(repository)
				.color(link_color(app.is_theme_dark()))
				.link(Message::OpenUrl(repository.to_string()))]
		),
	]
	.spacing(SPACING_AMOUNT)
	.width(Fill)
	.into()
}

fn item<'a>(
	label: impl Into<Element<'a, Message>>,
	content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
	row![label.into(), Space::new(Fill, 0.0), content.into(),]
		.spacing(SMALL_SPACING_AMOUNT)
		.align_y(Vertical::Center)
		.into()
}
