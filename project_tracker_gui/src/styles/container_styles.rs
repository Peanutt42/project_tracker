use iced::{widget::container::{Appearance, StyleSheet}, Border, Theme, Background};

use super::BORDER_RADIUS;

pub struct RoundedContainerStyle;

impl StyleSheet for RoundedContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(style.extended_palette().background.weak.color)),
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}