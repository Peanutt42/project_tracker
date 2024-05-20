use iced::{widget::{pick_list, column, row, container, text}, Element, Length, alignment::{Alignment, Horizontal, Vertical}};

use crate::{components::{horizontal_seperator, loading_screen}, styles::{LARGE_SPACING_AMOUNT, LARGE_TEXT_SIZE}};
use crate::styles::LARGE_PADDING_AMOUNT;
use crate::project_tracker::{ProjectTrackerApp, UiMessage};
use crate::theme_mode::ThemeMode;


pub struct SettingsPage {
	
}

impl SettingsPage {
	pub fn new() -> Self {
		Self {
			
		}
	}

	pub fn view<'a>(&'a self, app: &'a ProjectTrackerApp) -> Element<UiMessage> {
		if let Some(saved_state) = &app.saved_state {
			column![
				row![
					text("Theme Mode:").horizontal_alignment(Horizontal::Center).vertical_alignment(Vertical::Center),
					container(pick_list(&ThemeMode::ALL[..], Some(&saved_state.theme_mode), UiMessage::SetThemeMode))
						.width(Length::Fill)
						.align_x(Horizontal::Right),
				]
				.align_items(Alignment::Center),

				horizontal_seperator(1.0),

				column![
					text("Shortcuts").size(LARGE_TEXT_SIZE),
					text("Save: Ctrl + S"),
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