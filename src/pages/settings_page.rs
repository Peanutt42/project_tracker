use iced::{widget::{column, row, text}, Element};
use crate::{components::{horizontal_seperator, loading_screen, dangerous_button}, styles::{LARGE_PADDING_AMOUNT, SPACING_AMOUNT, LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE}};
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
			.into()
		}
		else {
			loading_screen()
		}
	}
}