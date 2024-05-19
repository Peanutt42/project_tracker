use iced::{Subscription, Theme, time};
use serde::{Serialize, Deserialize};

use crate::project_tracker::UiMessage;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
	#[default]
	System,
	Dark,
	Light
}

impl ThemeMode {
	pub const ALL: [ThemeMode; 3] = [
		ThemeMode::System,
		ThemeMode::Dark,
		ThemeMode::Light,
	];
}

impl std::fmt::Display for ThemeMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ThemeMode::System => f.write_str("System"),
			ThemeMode::Dark => f.write_str("Dark"),
			ThemeMode::Light => f.write_str("Light"),
		}
	}
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