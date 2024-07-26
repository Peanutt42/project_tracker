use iced::{color, widget::container::{Appearance, StyleSheet}, Border, Color, Shadow, Theme, Vector};
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
			// 25% weak color, 75% base color
			background: Some(
				mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color, 0.25).into()
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

pub const DROP_HIGHLIGHT_WIDTH: f32 = 2.0;

pub struct DropZoneContainerStyle {
	pub hovered: bool
}

impl StyleSheet for DropZoneContainerStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: None,
			border: Border {
				color: color!(0x3584e4), // highlight color
				width: if self.hovered { DROP_HIGHLIGHT_WIDTH } else { 0.0 },
				radius: BORDER_RADIUS.into()
			},
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: if self.hovered { 20.0 } else { 0.0 },
			},
			..Default::default()
		}
	}
}