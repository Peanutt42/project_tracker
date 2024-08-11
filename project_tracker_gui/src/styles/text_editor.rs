use iced::{widget::text_editor::{Appearance, StyleSheet}, Border, Color, Theme};

use super::BORDER_RADIUS;

pub struct TextEditorStyle {
	pub round_top_left: bool,
	pub round_top_right: bool,
	pub round_bottom_left: bool,
	pub round_bottom_right: bool,
}

impl StyleSheet for TextEditorStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			border: Border {
				radius: [
					if self.round_top_left { BORDER_RADIUS } else { 0.0 },
					if self.round_top_right { BORDER_RADIUS } else { 0.0 },
					if self.round_bottom_right { BORDER_RADIUS } else { 0.0 },
					if self.round_bottom_left { BORDER_RADIUS } else { 0.0 },
				].into(),
				width: 1.0,
				color: palette.background.strong.color,
			},
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			..self.active(style)
		}
	}

	fn focused(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			..self.active(style)
		}
	}

	fn disabled(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.weak.color.into(),
			..self.active(style)
		}
	}

	fn placeholder_color(&self, style: &Self::Style) -> Color {
		style.extended_palette().background.strong.color
	}

	fn value_color(&self, style: &Self::Style) -> Color {
		style.extended_palette().background.base.text
	}

	fn selection_color(&self, _style: &Self::Style) -> Color {
		use iced::color;
		color!(0x3367d1)
	}

	fn disabled_color(&self, style: &Self::Style) -> Color {
		self.placeholder_color(style)
	}
}