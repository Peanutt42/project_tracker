use iced::{alignment::Horizontal, widget::{column, container, row, text, Space}, Alignment, Task, Element, Length::Fill, Padding};
use iced_aw::card;
use crate::{components::{clear_synchronization_filepath_button, dangerous_button, export_database_button, file_location, filepath_widget, horizontal_seperator_padded, import_database_button, import_google_tasks_button, select_synchronization_filepath_button, settings_tab_button, sync_database_button, vertical_seperator, ErrorMsgModalMessage, HORIZONTAL_SCROLLABLE_PADDING}, integrations::{import_google_tasks_dialog, ImportGoogleTasksError}, styles::{card_style, GREY, PADDING_AMOUNT}};
use crate::core::{Database, DatabaseMessage, DateFormatting, PreferenceMessage, Preferences};
use crate::styles::{LARGE_TEXT_SIZE, SPACING_AMOUNT, rounded_container_style, HEADING_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};
use crate::icons::Bootstrap;

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

impl From<SettingsModalMessage> for UiMessage {
	fn from(value: SettingsModalMessage) -> Self {
		UiMessage::SettingsModalMessage(value)
	}
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum SettingTab {
	#[default]
	General,
	Database,
	Shortcuts,
}

impl SettingTab {
	fn view<'a>(&'a self, app: &'a ProjectTrackerApp, preferences: &'a Preferences) -> Element<'a, UiMessage> {
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

				column![
					shortcut("Open Settings:", "Ctrl + ,"),
					shortcut("Open Stopwatch:", "Ctrl + H"),
					shortcut("New Project:", "Ctrl + Shift + N"),
					shortcut("Rename Project:", "Ctrl + R"),
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
			},
		}
	}
}

#[derive(Debug, Clone, Default)]
pub enum SettingsModal {
	Opened {
		tab: SettingTab,
	},
	#[default]
	Closed,
}

impl SettingsModal {
	pub fn is_open(&self) -> bool {
		matches!(self, SettingsModal::Opened{ .. })
	}

	pub fn update(&mut self, message: SettingsModalMessage, preferences: &mut Option<Preferences>) -> Task<UiMessage> {
		match message {
			SettingsModalMessage::Open => {
				*self = SettingsModal::Opened{ tab: SettingTab::default() };
				Task::none()
			},
			SettingsModalMessage::Close => {
				*self = SettingsModal::Closed;
				Task::none()
			},

			SettingsModalMessage::BrowseSynchronizationFilepath => Task::perform(
				Database::export_file_dialog(),
				|filepath| {
					match filepath {
						Some(filepath) => PreferenceMessage::SetSynchronizationFilepath(Some(filepath)).into(),
						None => SettingsModalMessage::BrowseSynchronizationFilepathCanceled.into(),
					}
				}
			),
			SettingsModalMessage::BrowseSynchronizationFilepathCanceled => Task::none(),

			SettingsModalMessage::ImportGoogleTasksFileDialog => Task::perform(
				import_google_tasks_dialog(),
				move |result| {
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
				}
			),
			SettingsModalMessage::ImportGoogleTasksFileDialogCanceled => Task::none(),

			SettingsModalMessage::SetDateFormatting(date_formatting) => {
				if let Some(preferences) = preferences {
					preferences.update(PreferenceMessage::SetDateFormatting(date_formatting))
				}
				else {
					Task::none()
				}
			},

			SettingsModalMessage::SwitchSettingsTab(new_tab) => {
				if let SettingsModal::Opened { tab } = self {
					*tab = new_tab;
				}
				Task::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<Element<UiMessage>> {
		match self {
			SettingsModal::Closed => None,
			SettingsModal::Opened{ tab } => {
				if let Some(preferences) = &app.preferences {
					let tabs = column![
						settings_tab_button(SettingTab::General, *tab),
						settings_tab_button(SettingTab::Database, *tab),
						settings_tab_button(SettingTab::Shortcuts, *tab)
					]
					.spacing(SMALL_SPACING_AMOUNT)
					.padding(Padding{ right: PADDING_AMOUNT, ..Padding::ZERO });

					Some(
						card(
							text("Settings").size(HEADING_TEXT_SIZE),

							row![
								container(tabs)
									.width(150.0),

								vertical_seperator(),

								container(
									tab.view(app, preferences)
								)
								.center_x(Fill)
								.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
							]
						)
						.max_width(900.0)
						.max_height(400.0)
						.close_size(LARGE_TEXT_SIZE)
						.style(card_style)
						.on_close(SettingsModalMessage::Close.into())
						.into()
					)
				}
				else {
					None
				}
			},
		}
	}
}