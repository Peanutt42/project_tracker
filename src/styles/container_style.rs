use iced::{widget::container::{Appearance, StyleSheet}, Border, Background, Theme};
use crate::styles::{mix_color, BORDER_RADIUS};

pub struct HoverBackgroundContainerStyle {
	pub hovered: bool,
}

impl StyleSheet for HoverBackgroundContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: if self.hovered {
				Some(Background::Color(
					mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color)
				))
			}
			else {
				None
			},
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}
