use iced::{Subscription, Theme, time};
use serde::{Serialize, Deserialize};

use crate::project_tracker::UiMessage;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ThemeMode {
	#[default]
	System,
	Dark,
	Light
}

pub fn get_theme(dark_mode: bool) -> Theme {
	if dark_mode {
		Theme::Dark
	}
	else {
		Theme::Light
	}
}

pub fn is_system_theme_dark() -> bool {
	match dark_light::detect() {
		dark_light::Mode::Dark | dark_light::Mode::Default => true,
		dark_light::Mode::Light => false,
	}
}

pub fn system_theme_subscription() -> Subscription<UiMessage> {
	time::every(time::Duration::from_secs(1))
				.map(|_| UiMessage::SystemTheme{ is_dark: is_system_theme_dark() })
}