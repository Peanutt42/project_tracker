use crate::styles::GREY;
use iced::{
	theme::{
		palette::{Extended, Secondary},
		Custom, Palette,
	},
	Color, Theme,
};
use std::sync::{Arc, LazyLock};

#[derive(Copy, Clone, Default)]
pub enum ProjectTrackerTheme {
	#[default]
	Dark,
	Light,
}

pub static DARK_THEME: LazyLock<Theme> = LazyLock::new(|| {
	let palette = Palette {
		background: Color::from_rgb(0.05, 0.05, 0.05),
		text: Color::from_rgb(0.95, 0.95, 0.95),
		primary: Color::from_rgb8(0, 41, 229),
		success: Color::from_rgb8(48, 211, 48),
		danger: Color::from_rgb(0.9, 0.0, 0.0),
	};

	Theme::Custom(Arc::new(Custom::with_fn(
		"Dark".to_string(),
		palette,
		|p| Extended {
			secondary: Secondary::generate(p.background, GREY),
			..Extended::generate(p)
		},
	)))
});

pub static LIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| {
	let palette = Palette {
		background: Color::from_rgb(0.95, 0.95, 0.95),
		text: Color::from_rgb(0.05, 0.05, 0.05),
		primary: Color::from_rgb8(0, 40, 219),
		success: Color::from_rgb8(48, 211, 48),
		danger: Color::from_rgb(0.9, 0.0, 0.0),
	};

	Theme::Custom(Arc::new(Custom::with_fn(
		"Light".to_string(),
		palette,
		|p| Extended {
			secondary: Secondary::generate(p.background, GREY),
			..Extended::generate(p)
		},
	)))
});

impl ProjectTrackerTheme {
	pub fn get_theme(self) -> &'static Theme {
		match self {
			ProjectTrackerTheme::Dark => &DARK_THEME,
			ProjectTrackerTheme::Light => &LIGHT_THEME,
		}
	}
}
