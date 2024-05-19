use iced::{widget::{pick_list, row, container, text}, Element, Length, alignment::{Alignment, Horizontal, Vertical}};

use crate::components::loading_screen;
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
			row![
				text("Theme Mode:").horizontal_alignment(Horizontal::Center).vertical_alignment(Vertical::Center),
				container(pick_list(&ThemeMode::ALL[..], Some(&saved_state.theme_mode), UiMessage::SetThemeMode))
					.width(Length::Fill)
					.align_x(Horizontal::Right)
			]
			.padding(LARGE_PADDING_AMOUNT)
			.align_items(Alignment::Center)
			.into()
		}
		else {
			loading_screen()
		}
	}
}