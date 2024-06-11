use iced::{theme, widget::{column, row, scrollable, text}, Element};
use crate::components::{dangerous_button, horizontal_seperator, loading_screen};
use crate::styles::{scrollable_vertical_direction, ScrollableStyle, LARGE_PADDING_AMOUNT, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE, SPACING_AMOUNT};
use crate::project_tracker::{ProjectTrackerApp, UiMessage};


pub struct SettingsPage {

}

impl SettingsPage {
	pub fn new() -> Self {
		Self {

		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(preferences) = &app.preferences {
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
							dangerous_button("Clear Database")
								.on_press(UiMessage::ClearDatabase),
							dangerous_button("Import Database")
								.on_press(UiMessage::ImportDatabase),
							dangerous_button("Export Database")
								.on_press(UiMessage::ExportDatabase),
						]
						.spacing(SPACING_AMOUNT)
					],

					horizontal_seperator(),

					column![
						text("Shortcuts").size(LARGE_TEXT_SIZE),
						text("New Project: Ctrl + Shift + N"),
						text("New Task: Ctrl + N"),
					]
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
