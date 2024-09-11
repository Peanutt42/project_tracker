use iced::{Subscription, Theme, time};
use serde::{Serialize, Deserialize};
use crate::project_tracker::UiMessage;
use crate::styles::ProjectTrackerTheme;

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
	#[default]
	System,
	Dark,
	Light
}

pub fn get_theme(dark_mode: bool) -> &'static Theme {
	if dark_mode {
		ProjectTrackerTheme::Dark.get_theme()
	}
	else {
		ProjectTrackerTheme::Light.get_theme()
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
