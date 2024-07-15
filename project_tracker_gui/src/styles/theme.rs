use std::sync::Arc;
use iced::{theme::{palette::{Extended, Secondary}, Custom, Palette}, Color, Theme};
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
			ProjectTrackerTheme::Dark => {
				let palette = Palette {
					background: Color::from_rgb(0.1, 0.1, 0.1),
					text: Color::from_rgb(0.9, 0.9, 0.9),
					primary: NICE_GREEN,
					success: LIGHT_DARK_GREEN,
					danger: Color::from_rgb(1.0, 0.0, 0.0),
				};

				Theme::Custom(Arc::new(Custom::with_fn("Dark".to_string(), palette, |p| {
					Extended {
						secondary: Secondary::generate(p.background, Color::from_rgb(0.5, 0.5, 0.5)),
						..Extended::generate(palette)
					}
				})))
			},
			ProjectTrackerTheme::Light => Theme::custom(
				"Light".to_string(),
				Palette {
					background: Color::WHITE,
					text: Color::BLACK,
					primary: NICE_GREEN,
					success: LIGHT_DARK_GREEN,
					danger: Color::from_rgb(1.0, 0.0, 0.0),
				}
			),
		}
	}
}
