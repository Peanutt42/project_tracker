use iced::{widget::container::{Appearance, StyleSheet}, Background, Border, Color, Shadow, Theme, Vector};

pub struct ContextMenuContainerStyle;

impl StyleSheet for ContextMenuContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			text_color: None,
			background: Some(Background::Color(style.extended_palette().background.base.color)),
			border: Border::with_radius(7.5),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::ZERO,
				blur_radius: 20.0,
			}
		}
	}
}