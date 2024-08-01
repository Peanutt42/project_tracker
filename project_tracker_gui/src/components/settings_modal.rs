use iced::{theme, widget::{button, column, container, row, text}, Alignment, Command, Element};
use iced_aw::{Bootstrap, CardStyles, ModalStyles, card};
use crate::{components::{dangerous_button, file_location, horizontal_seperator, sync_database_button}, core::PreferenceMessage, styles::{capped_text, ModalCardStyle, ModalStyle, RoundedContainerStyle, RoundedSecondaryButtonStyle, HEADING_TEXT_SIZE, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT}};
use crate::core::{Database, DatabaseMessage};
use crate::styles::{LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub enum SettingsModalMessage {
	Open,
	Close,

	BrowseSynchronizationFilepath,
	BrowseSynchronizationFilepathCanceled,
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
		matches!(self, SettingsModal::Opened)
	}

	pub fn update(&mut self, message: SettingsModalMessage) -> Command<UiMessage> {
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
				Database::import_file_dialog(),
				|filepath| {
					match filepath {
						Some(filepath) => PreferenceMessage::SetSynchronizationFilepath(Some(filepath)).into(),
						None => SettingsModalMessage::BrowseSynchronizationFilepathCanceled.into(),
					}
				}
			),
			SettingsModalMessage::BrowseSynchronizationFilepathCanceled => Command::none(),
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
										file_location(&Database::get_filepath()),
									]
									.align_items(Alignment::Center),

									row![
										text("Synchronization file location: "),
										container(
											text(
												if let Some(filepath) = preferences.synchronization_filepath() {
													capped_text(&filepath.to_string_lossy(), 60)
												}
												else {
													"not specified".to_string()
												}
											)
										)
										.style(theme::Container::Box),
										button(text("Clear"))
											.on_press(PreferenceMessage::SetSynchronizationFilepath(None).into())
											.style(theme::Button::custom(RoundedSecondaryButtonStyle)),
										button(text("Browse"))
											.on_press(SettingsModalMessage::BrowseSynchronizationFilepath.into())
											.style(theme::Button::custom(RoundedSecondaryButtonStyle)),
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
							.padding(LARGE_PADDING_AMOUNT)
							.spacing(LARGE_SPACING_AMOUNT)
						)
						.max_width(900.0)
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