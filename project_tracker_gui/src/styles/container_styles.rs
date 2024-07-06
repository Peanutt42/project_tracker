use iced::{widget::container::{Appearance, StyleSheet}, Border, Color, Shadow, Theme, Vector};
use crate::styles::{BORDER_RADIUS, LARGE_BORDER_RADIUS, mix_color};

pub struct RoundedContainerStyle;

impl StyleSheet for RoundedContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().background.weak.color.into()),
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}

pub struct PaletteContainerStyle;

impl StyleSheet for PaletteContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			// 75% base color, 25% weak color
			background: Some(
				mix_color(
					mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color),
					style.extended_palette().background.base.color
				).into()
			),
			border: Border::with_radius(LARGE_BORDER_RADIUS),
			shadow: Shadow {
				blur_radius: if style.extended_palette().is_dark { 50.0 } else { 35.0 },
				color: if style.extended_palette().is_dark { Color::BLACK } else { Color::from_rgba(0.0, 0.0, 0.0, 0.5) },
				offset: Vector::ZERO,
			},
			..Default::default()
		}
	}
}