use iced::{Color, widget::progress_bar::{StyleSheet, Appearance}, Background, Theme};

pub struct CompletionBarStyle;

impl StyleSheet for CompletionBarStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Background::Color(Color::from_rgb(0.25, 0.25, 0.25)),
			bar: Background::Color(Color::from_rgb(0.0, 1.0, 0.0)),
			border_radius: 2.5.into(),
		}
	}
}
