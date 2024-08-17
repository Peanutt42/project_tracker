use iced::{alignment::Horizontal, theme, widget::{column, container, row, text}, Alignment, Command, Element, Length};
use iced_aw::{card, Bootstrap, CardStyles, ModalStyles};
use crate::components::{dangerous_button, file_location, filepath_widget, horizontal_seperator, sync_database_button, select_synchronization_filepath_button, clear_synchronization_filepath_button};
use crate::core::{Database, DatabaseMessage, DateFormatting, PreferenceMessage, Preferences};
use crate::styles::{LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SPACING_AMOUNT, ModalCardStyle, ModalStyle, RoundedContainerStyle, HEADING_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub enum SettingsModalMessage {
	Open,
	Close,

	BrowseSynchronizationFilepath,
	BrowseSynchronizationFilepathCanceled,

	SetDateFormatting(DateFormatting),
}

impl From<SettingsModalMessage> for UiMessage {
	fn from(value: SettingsModalMessage) -> Self {
		UiMessage::SettingsModalMessage(value)
	}
}

#[derive(Debug, Clone, Default)]
pub enum SettingsModal {
	Opened,
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
				*self = SettingsModal::Opened;
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
			}
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Option<(Element<UiMessage>, ModalStyles)> {
		match self {
			SettingsModal::Closed => None,
			SettingsModal::Opened => {
				if let Some(preferences) = &app.preferences {
					let shortcut = |name, shortcut| {
						row![
							text(name),
							container(
								container(text(shortcut)).padding(SMALL_HORIZONTAL_PADDING)
							)
							.style(theme::Container::Custom(Box::new(RoundedContainerStyle)))
						]
						.spacing(SMALL_SPACING_AMOUNT)
					};

					Some((
						card(
							text("Settings").size(HEADING_TEXT_SIZE),

							column![
								column![
									text("Preferences").size(LARGE_TEXT_SIZE),

									preferences.view(),
								],

								horizontal_seperator(),

								column![
									text("Database").size(LARGE_TEXT_SIZE),

									row![
										text("File location: "),
										container(file_location(&Database::get_filepath()))
											.width(Length::Fill)
											.align_x(Horizontal::Right),
									]
									.align_items(Alignment::Center),

									row![
										text("Synchronization file location: "),

										if let Some(filepath) = preferences.synchronization_filepath() {
											filepath_widget(filepath)
										}
										else {
											"not specified".into()
										},

										clear_synchronization_filepath_button(),

										select_synchronization_filepath_button(),
									]
									.spacing(SPACING_AMOUNT)
									.align_items(Alignment::Center),

									sync_database_button(app.database.as_ref().map(|db| db.is_syncing()).unwrap_or(false), preferences.synchronization_filepath().clone()),

									row![
										dangerous_button(
											Bootstrap::Trash,
											"Clear",
											Some("Clear Database".to_string()),
											DatabaseMessage::Clear
										),
										dangerous_button(
											Bootstrap::Download,
											"Import",
											None,
											DatabaseMessage::ImportDialog
										),
										dangerous_button(
											Bootstrap::Upload,
											"Export",
											None,
											DatabaseMessage::ExportDialog
										),
									]
									.spacing(SPACING_AMOUNT),
								]
								.spacing(SPACING_AMOUNT),

								horizontal_seperator(),

								column![
									text("Shortcuts").size(LARGE_TEXT_SIZE),
									shortcut("Open Settings:", "Ctrl + ,"),
									shortcut("Open Overview:", "Ctrl + H"),
									shortcut("New Project:", "Ctrl + Shift + N"),
									shortcut("Rename Project:", "Ctrl + R"),
									shortcut("Delete Project:", "Ctrl + Del"),
									shortcut("Switch to lower Project:", "Ctrl + Tab"),
									shortcut("Switch to upper Project:", "Ctrl + Shift + Tab"),
									shortcut("New Task:", "Ctrl + N"),
									shortcut("Toggle Sidebar:", "Ctrl + B"),
								]
								.spacing(SMALL_SPACING_AMOUNT)
							]
							//.padding(LARGE_PADDING_AMOUNT)
							.spacing(LARGE_SPACING_AMOUNT)
						)
						.max_width(600.0)
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