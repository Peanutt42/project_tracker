use iced::{widget::text_input::{Appearance, StyleSheet}, Border, Color, Theme};

use super::BORDER_RADIUS;

pub struct TextInputStyle {
	pub round_left: bool,
	pub round_right: bool,
}

impl StyleSheet for TextInputStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			border: Border {
				radius: [
					if self.round_left { BORDER_RADIUS } else { 0.0 },
					if self.round_right { BORDER_RADIUS } else { 0.0 },
					if self.round_right { BORDER_RADIUS } else { 0.0 },
					if self.round_left { BORDER_RADIUS } else { 0.0 },
				].into(),
				width: 1.0,
				color: palette.background.strong.color,
			},
			icon_color: palette.background.weak.text,
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			icon_color: palette.background.weak.text,
			..self.active(style)
		}
	}

	fn focused(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			icon_color: palette.background.weak.text,
			..self.active(style)
		}
	}

	fn disabled(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.weak.color.into(),
			icon_color: palette.background.strong.color,
			..self.active(style)
		}
	}

	fn placeholder_color(&self, style: &Self::Style) -> Color {
		style.extended_palette().background.strong.color
	}

	fn value_color(&self, style: &Self::Style) -> Color {
		style.extended_palette().background.base.text
	}

	fn selection_color(&self, style: &Self::Style) -> Color {
		style.extended_palette().primary.weak.color
	}

	fn disabled_color(&self, style: &Self::Style) -> Color {
		self.placeholder_color(style)
	}
}