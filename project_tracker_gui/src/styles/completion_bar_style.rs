use iced::{widget::progress_bar::{StyleSheet, Appearance}, Theme};
use crate::styles::{NICE_GREEN, DARK_GREY};

pub struct CompletionBarStyle;

impl StyleSheet for CompletionBarStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: DARK_GREY.into(),
			bar: NICE_GREEN.into(),
			border_radius: 2.5.into(),
		}
	}
}
