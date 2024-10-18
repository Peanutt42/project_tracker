use crate::core::{Database, DatabaseMessage, DateFormatting, PreferenceAction, PreferenceMessage, Preferences};
use crate::icons::Bootstrap;
use crate::project_tracker::{ProjectTrackerApp, Message};
use crate::styles::{
	rounded_container_style, HEADING_TEXT_SIZE, LARGE_TEXT_SIZE, SMALL_HORIZONTAL_PADDING,
	SMALL_SPACING_AMOUNT, SPACING_AMOUNT,
};
use crate::{
	components::{
		clear_synchronization_filepath_button, dangerous_button, export_database_button,
		file_location, filepath_widget, horizontal_seperator_padded, import_database_button,
		import_google_tasks_button, select_synchronization_filepath_button, settings_tab_button,
		sync_database_button, vertical_seperator, copy_to_clipboard_button, open_link_button,
		HORIZONTAL_SCROLLABLE_PADDING,
	},
	modals::ErrorMsgModalMessage,
	integrations::{import_google_tasks_dialog, ImportGoogleTasksError},
	styles::{card_style, GREY, PADDING_AMOUNT},
};
use iced::alignment::Vertical;
use iced::{
	alignment::Horizontal,
	keyboard,
	padding::{left, right},
	widget::{column, container, row, text, Column, Space},
	Alignment, Element,
	Length::Fill,
	Padding, Subscription, Task,
};
use iced_aw::card;

#[derive(Debug, Clone)]
pub enum SettingsModalMessage {
	Open,
	Close,
	SwitchSettingsTab(SettingTab),

	SetDateFormatting(DateFormatting),

	BrowseSynchronizationFilepath,
	BrowseSynchronizationFilepathCanceled,

	ImportGoogleTasksFileDialog,
	ImportGoogleTasksFileDialogCanceled,
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
	About,
}

impl SettingTab {
	const ALL: [SettingTab; 4] = [
		SettingTab::General,
		SettingTab::Database,
		SettingTab::Shortcuts,
		SettingTab::About,
	];

	fn view<'a>(
		&'a self,
		app: &'a ProjectTrackerApp,
		preferences: &'a Preferences,
	) -> Element<'a, Message> {
		match self {
			SettingTab::General => preferences.view(),
			SettingTab::Database => {
				column![
					row![
						container("Database file location: ").padding(HORIZONTAL_SCROLLABLE_PADDING),
						container(file_location(Database::get_filepath()))
							.width(Fill)
							.align_x(Horizontal::Right),
					]
					.align_y(Alignment::Center),

					container(
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
						.spacing(SPACING_AMOUNT)
					)
					.width(Fill)
					.align_x(Horizontal::Right),

					horizontal_seperator_padded(),

					row![
						container("Synchronization file location: ")
							.padding(HORIZONTAL_SCROLLABLE_PADDING),

						if let Some(filepath) = preferences.synchronization_filepath() {
							Element::new(filepath_widget(filepath.clone()).width(Fill))
						}
						else {
							"not specified".into()
						},

						container(clear_synchronization_filepath_button())
							.padding(HORIZONTAL_SCROLLABLE_PADDING),

						container(select_synchronization_filepath_button())
							.padding(HORIZONTAL_SCROLLABLE_PADDING),
					]
					.spacing(SPACING_AMOUNT)
					.align_y(Alignment::Center),

					container(
						sync_database_button(app.syncing_database, preferences.synchronization_filepath().clone())
					)
					.width(Fill)
					.align_x(Horizontal::Right),

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
							text("Go to https://myaccount.google.com/dashboard and download the Tasks data.\nThen extract the Takeout.zip and import the \"Tasks.json\" file inside under the \"Tasks\" folder.")
								.style(|_theme| text::Style{ color: Some(GREY) })
						)
						.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
					]
					.spacing(SPACING_AMOUNT),
				]
				.spacing(SPACING_AMOUNT)
				.into()
			},
			SettingTab::Shortcuts => {
				let shortcut = |name, shortcut| {
					row![
						text(name),
						Space::new(Fill, 0.0),
						container(
							container(shortcut).padding(SMALL_HORIZONTAL_PADDING)
						)
						.style(rounded_container_style)
					]
					.spacing(SMALL_SPACING_AMOUNT)
				};

				container(
					column![
						shortcut("Open Settings:", "Ctrl + ,"),
						shortcut("Open Stopwatch:", "Ctrl + H"),
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
				)
				.center_y(Fill)
				.into()
			},
			SettingTab::About => {
				let item = |label: &'static str, content: Element<'static, Message>| {
					row![
						label,
						Space::new(Fill, 0.0),
						content,
					]
					.spacing(SMALL_SPACING_AMOUNT)
				};

				let repository = env!("CARGO_PKG_REPOSITORY");

				let author_link = "https://github.com/Peanutt42";

				column![
					item("Project Tracker:", text("Project Todo Tracker for personal programming projects").into()),

					item(
						"Author:",
						row![
							text("P3anutt42 (github)"),

							copy_to_clipboard_button(author_link.to_string()),

							open_link_button(author_link.to_string()),
						]
						.spacing(SMALL_SPACING_AMOUNT)
						.align_y(Vertical::Center)
						.into()
					),

					item("Version:", text(env!("CARGO_PKG_VERSION")).into()),

					item(
						"Repository:",
						row![
							text(repository),

							copy_to_clipboard_button(repository.to_string()),

							open_link_button(repository.to_string()),
						]
						.spacing(SMALL_SPACING_AMOUNT)
						.align_y(Vertical::Center)
						.into()
					),
				]
				.spacing(SPACING_AMOUNT)
				.width(Fill)
				.into()
			},
		}
	}
}

#[derive(Debug, Clone, Default)]
pub enum SettingsModal {
	Opened {
		selected_tab: SettingTab,
	},
	#[default]
	Closed,
}

impl SettingsModal {
	pub fn is_open(&self) -> bool {
		matches!(self, SettingsModal::Opened { .. })
	}

	pub fn subscription(&self) -> Subscription<SettingsModalMessage> {
		keyboard::on_key_press(|key, modifiers| match key.as_ref() {
			keyboard::Key::Character(",") if modifiers.command() => {
				Some(SettingsModalMessage::Open)
			}
			_ => None,
		})
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
				};
				PreferenceAction::None
			}
			SettingsModalMessage::Close => {
				*self = SettingsModal::Closed;
				PreferenceAction::None
			}

			SettingsModalMessage::BrowseSynchronizationFilepath => {
				Task::perform(Database::export_file_dialog(), |filepath| match filepath {
					Some(filepath) => {
						PreferenceMessage::SetSynchronizationFilepath(Some(filepath)).into()
					}
					None => SettingsModalMessage::BrowseSynchronizationFilepathCanceled.into(),
				})
				.into()
			}
			SettingsModalMessage::BrowseSynchronizationFilepathCanceled => PreferenceAction::None,

			SettingsModalMessage::ImportGoogleTasksFileDialog => {
				Task::perform(import_google_tasks_dialog(), move |result| {
					match result {
						Some((result, filepath)) => match result {
							Ok(projects) => DatabaseMessage::ImportProjects(projects).into(),
							Err(import_error) => match import_error {
								ImportGoogleTasksError::IoError(io_error) => ErrorMsgModalMessage::open(
									format!("Failed to open google tasks takeout .json file\nLocation: {}\nError: {io_error}", filepath.display())
								),
								ImportGoogleTasksError::ParseError(parse_error) => ErrorMsgModalMessage::open(
									format!("Failed to parse google tasks takeout .json file\nLocation: {}\nError: {parse_error}", filepath.display())
								),
							}
						},
						None => SettingsModalMessage::BrowseSynchronizationFilepathCanceled.into(),
					}
				})
				.into()
			}
			SettingsModalMessage::ImportGoogleTasksFileDialogCanceled => PreferenceAction::None,

			SettingsModalMessage::SetDateFormatting(date_formatting) => {
				if let Some(preferences) = preferences {
					preferences.update(PreferenceMessage::SetDateFormatting(date_formatting))
				} else {
					PreferenceAction::None
				}
			}

			SettingsModalMessage::SwitchSettingsTab(new_tab) => {
				if let SettingsModal::Opened { selected_tab } = self {
					*selected_tab = new_tab;
				}
				PreferenceAction::None
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<Element<Message>> {
		match self {
			SettingsModal::Closed => None,
			SettingsModal::Opened { selected_tab } => app.preferences.as_ref().map(|preferences| {
				let tabs: Vec<Element<Message>> = SettingTab::ALL
					.iter()
					.map(|tab| settings_tab_button(*tab, *selected_tab).into())
					.collect();

				card(
					text("Settings").size(HEADING_TEXT_SIZE),
					row![
						Column::with_children(tabs)
							.width(150.0)
							.spacing(SMALL_SPACING_AMOUNT)
							.padding(right(PADDING_AMOUNT)),
						vertical_seperator(),
						container(selected_tab.view(app, preferences))
							.width(Fill)
							.padding(left(PADDING_AMOUNT))
					],
				)
				.max_width(900.0)
				.max_height(475.0)
				.close_size(LARGE_TEXT_SIZE)
				.on_close(SettingsModalMessage::Close.into())
				.style(card_style)
				.into()
			}),
		}
	}
}
