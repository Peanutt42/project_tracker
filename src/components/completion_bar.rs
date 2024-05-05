use iced::{Color, widget::{progress_bar, progress_bar::{StyleSheet, Appearance}, ProgressBar}, Background, Theme};

pub fn completion_bar(completion: f32) -> ProgressBar {
	progress_bar(0.0..=100.0, completion * 100.0)
		.style(iced::theme::ProgressBar::Custom(Box::new(CompletionBarStyle)))
		.height(5.0)
}

struct CompletionBarStyle;

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
