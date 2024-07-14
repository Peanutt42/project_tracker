use iced::{theme, widget::{container, column, row, scrollable, text}, Element, Alignment};
use crate::{components::{dangerous_button, file_location, horizontal_seperator, loading_screen}, styles::{RoundedContainerStyle, SMALL_HORIZONTAL_PADDING, SMALL_SPACING_AMOUNT}};
use crate::core::{Database, DatabaseMessage};
use crate::styles::{scrollable_vertical_direction, ScrollableStyle, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};


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
							dangerous_button("Clear Database", DatabaseMessage::Clear),
							dangerous_button("Import Database", DatabaseMessage::Import),
							dangerous_button("Export Database", DatabaseMessage::Export),
						]
						.spacing(SPACING_AMOUNT),

						row![
							text("Database file location: "),
							file_location(Database::get_filepath()),
						]
						.align_items(Alignment::Center),
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