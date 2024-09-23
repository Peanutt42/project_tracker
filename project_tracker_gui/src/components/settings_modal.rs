use iced::{alignment::Horizontal, theme, widget::{column, container, row, text, Space}, Alignment, Command, Element, Length, Padding};
use iced_aw::{card, Bootstrap, CardStyles, ModalStyles};
use crate::{components::{clear_synchronization_filepath_button, dangerous_button, import_database_button, export_database_button, file_location, filepath_widget, vertical_seperator, select_synchronization_filepath_button, settings_tab_button, sync_database_button, horizontal_seperator_padded, HORIZONTAL_SCROLLABLE_PADDING}, styles::PADDING_AMOUNT};
use crate::core::{Database, DatabaseMessage, DateFormatting, PreferenceMessage, Preferences};
use crate::styles::{LARGE_TEXT_SIZE, SPACING_AMOUNT, ModalCardStyle, ModalStyle, RoundedContainerStyle, HEADING_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub enum SettingsModalMessage {
	Open,
	Close,

	BrowseSynchronizationFilepath,
	BrowseSynchronizationFilepathCanceled,

	SetDateFormatting(DateFormatting),

	SwitchSettingsTab(SettingTab),
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
						container(file_location(&Database::get_filepath()))
							.width(Length::Fill)
							.align_x(Horizontal::Right),
					]
					.align_items(Alignment::Center),

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
					.width(Length::Fill)
					.align_x(Horizontal::Right),

					horizontal_seperator_padded(),

					row![
						container("Synchronization file location: ")
							.padding(HORIZONTAL_SCROLLABLE_PADDING),

						if let Some(filepath) = preferences.synchronization_filepath() {
							Element::new(filepath_widget(filepath).width(Length::Fill))
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
					.align_items(Alignment::Center),

					container(
						sync_database_button(app.syncing_database, preferences.synchronization_filepath().clone())
					)
					.width(Length::Fill)
					.align_x(Horizontal::Right),
				]
				.spacing(SPACING_AMOUNT)
				.into()
			},
			SettingTab::Shortcuts => {
				let shortcut = |name, shortcut| {
					row![
						text(name),
						Space::new(Length::Fill, 0.0),
						container(
							container(shortcut).padding(SMALL_HORIZONTAL_PADDING)
						)
						.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
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

	pub fn update(&mut self, message: SettingsModalMessage, preferences: &mut Option<Preferences>) -> Command<UiMessage> {
		match message {
			SettingsModalMessage::Open => {
				*self = SettingsModal::Opened{ tab: SettingTab::default() };
				Command::none()
			},
			SettingsModalMessage::Close => {
				*self = SettingsModal::Closed;
				Command::none()
			},

			SettingsModalMessage::BrowseSynchronizationFilepath => Command::perform(
				Database::export_file_dialog(),
				|filepath| {
					match filepath {
						Some(filepath) => PreferenceMessage::SetSynchronizationFilepath(Some(filepath)).into(),
						None => SettingsModalMessage::BrowseSynchronizationFilepathCanceled.into(),
					}
				}
			),
			SettingsModalMessage::BrowseSynchronizationFilepathCanceled => Command::none(),

			SettingsModalMessage::SetDateFormatting(date_formatting) => {
				if let Some(preferences) = preferences {
					preferences.update(PreferenceMessage::SetDateFormatting(date_formatting))
				}
				else {
					Command::none()
				}
			},

			SettingsModalMessage::SwitchSettingsTab(new_tab) => {
				if let SettingsModal::Opened { tab } = self {
					*tab = new_tab;
				}
				Command::none()
			},
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<(Element<UiMessage>, ModalStyles)> {
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

					Some((
						card(
							text("Settings").size(HEADING_TEXT_SIZE),

							row![
								container(tabs)
									.width(150.0),

								vertical_seperator(),

								container(
									tab.view(app, preferences)
								)
								.width(Length::Fill)
								.center_x()
								.padding(Padding{ left: PADDING_AMOUNT, ..Padding::ZERO })
							]
						)
						.max_width(900.0)
						.max_height(400.0)
						.close_size(LARGE_TEXT_SIZE)
						.style(CardStyles::custom(ModalCardStyle))
						.on_close(SettingsModalMessage::Close.into())
						.into(),

						ModalStyles::custom(ModalStyle)
					))
				}
				else {
					None
				}
			},
		}
	}
}