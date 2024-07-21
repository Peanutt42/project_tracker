use iced::{theme, widget::{button, column, container, row, scrollable, text}, Alignment, Command, Element};
use iced_aw::Bootstrap;
use crate::{components::{dangerous_button, file_location, horizontal_seperator, loading_screen, sync_database_button}, core::PreferenceMessage, styles::{RoundedContainerStyle, RoundedSecondaryButtonStyle, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT}};
use crate::core::{Database, DatabaseMessage};
use crate::styles::{scrollable_vertical_direction, ScrollableStyle, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};

#[derive(Debug, Clone)]
pub enum SettingsPageMessage {
	BrowseSynchronizationFilepath,
	BrowseSynchronizationFilepathCanceled,
}

impl From<SettingsPageMessage> for UiMessage {
	fn from(value: SettingsPageMessage) -> Self {
		UiMessage::SettingsPageMessage(value)
	}
}

pub struct SettingsPage {

}

impl Default for SettingsPage {
	fn default() -> Self {
		Self::new()
	}
}

impl SettingsPage {
	pub fn new() -> Self {
		Self {

		}
	}

	pub fn update(&mut self, message: SettingsPageMessage) -> Command<UiMessage> {
		match message {
			SettingsPageMessage::BrowseSynchronizationFilepath => Command::perform(
				Database::import_file_dialog(),
				|filepath| {
					match filepath {
						Some(filepath) => PreferenceMessage::SetSynchronizationFilepath(Some(filepath)).into(),
						None => SettingsPageMessage::BrowseSynchronizationFilepathCanceled.into(),
					}
				}
			),
			SettingsPageMessage::BrowseSynchronizationFilepathCanceled => Command::none(),
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
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

			scrollable(
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
									if let Some(filepath) = &preferences.synchronization_filepath {
										format!("{}", filepath.display())
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
								.on_press(SettingsPageMessage::BrowseSynchronizationFilepath.into())
								.style(theme::Button::custom(RoundedSecondaryButtonStyle)),
						]
						.spacing(SPACING_AMOUNT)
						.align_items(Alignment::Center),

						sync_database_button(app.database.as_ref().map(|db| db.syncing).unwrap_or(false), preferences.synchronization_filepath.clone()),

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
			.direction(scrollable_vertical_direction())
			.style(theme::Scrollable::custom(ScrollableStyle))
			.into()
		}
		else {
			loading_screen()
		}
	}
}