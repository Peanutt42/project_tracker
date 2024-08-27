use iced::{widget::progress_bar::{StyleSheet, Appearance}, Theme};
use crate::styles::color_average;

pub struct CompletionBarStyle;

impl StyleSheet for CompletionBarStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: color_average(
				style.extended_palette().background.weak.color,
				style.extended_palette().background.base.color
			)
			.into(),
			bar: style.extended_palette().primary.base.color.into(),
			border_radius: 2.5.into(),
		}
	}
}
