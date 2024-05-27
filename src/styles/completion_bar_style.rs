use iced::{widget::progress_bar::{StyleSheet, Appearance}, Background, Theme};
use crate::styles::{NICE_GREEN, DARK_GREY};

pub struct CompletionBarStyle;

impl StyleSheet for CompletionBarStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Background::Color(DARK_GREY),
			bar: Background::Color(NICE_GREEN),
			border_radius: 2.5.into(),
		}
	}
}
