use crate::project_tracker::Message;
use crate::styles::ProjectTrackerTheme;
use iced::{time, Subscription, Theme};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
	#[default]
	System,
	Dark,
	Light,
}

pub fn get_theme(dark_mode: bool) -> &'static Theme {
	if dark_mode {
		ProjectTrackerTheme::Dark.get_theme()
	} else {
		ProjectTrackerTheme::Light.get_theme()
	}
}

pub fn is_system_theme_dark() -> bool {
	match dark_light::detect() {
		Ok(mode) => match mode {
			dark_light::Mode::Dark | dark_light::Mode::Unspecified => true,
			dark_light::Mode::Light => false,
		},
		Err(_) => true,
	}
}

pub fn system_theme_subscription() -> Subscription<Message> {
	time::every(time::Duration::from_secs(1)).map(|_| Message::SystemTheme {
		is_dark: is_system_theme_dark(),
	})
}
