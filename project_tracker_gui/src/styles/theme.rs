use iced::{theme::Palette, Color, Theme};
use crate::styles::{LIGHT_DARK_GREEN, NICE_GREEN};


#[derive(Copy, Clone, Default)]
pub enum ProjectTrackerTheme {
	#[default]
	Dark,
	Light,
}

impl ProjectTrackerTheme {
	pub fn get_theme(self) -> Theme {
		match self {
			ProjectTrackerTheme::Dark => Theme::custom(
				"Dark".to_string(),
				Palette {
					background: Color::from_rgb(0.1, 0.1, 0.1),
					text: Color::from_rgb(0.9, 0.9, 0.9),
					primary: NICE_GREEN,
					success: LIGHT_DARK_GREEN,
					danger: Color::from_rgb(1.0, 0.0, 0.0),
				}
			),
			ProjectTrackerTheme::Light => Theme::Light,
		}
	}
}
