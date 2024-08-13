use iced::{widget::text_input::{Appearance, StyleSheet}, Border, Color, Theme};

use super::BORDER_RADIUS;

pub struct TextInputStyle {
	pub round_left_top: bool,
	pub round_right_top: bool,
	pub round_right_bottom: bool,
	pub round_left_bottom: bool,
}

impl TextInputStyle {
	pub const ONLY_ROUND_LEFT: Self = Self {
		round_left_top: true,
		round_right_top: false,
		round_right_bottom: false,
		round_left_bottom: true,
	};

	pub const NO_ROUNDING: Self = Self {
		round_left_top: false,
		round_right_top: false,
		round_right_bottom: false,
		round_left_bottom: false,
	};
}

impl Default for TextInputStyle {
	fn default() -> Self {
		Self {
			round_left_top: true,
			round_right_top: true,
			round_right_bottom: true,
			round_left_bottom: true,
		}
	}
}

impl StyleSheet for TextInputStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
			background: palette.background.base.color.into(),
			border: Border {
				radius: [
					if self.round_left_top { BORDER_RADIUS } else { 0.0 },
					if self.round_right_top { BORDER_RADIUS } else { 0.0 },
					if self.round_right_bottom { BORDER_RADIUS } else { 0.0 },
					if self.round_left_bottom { BORDER_RADIUS } else { 0.0 },
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

	fn selection_color(&self, _style: &Self::Style) -> Color {
		use iced::color;
		color!(0x3367d1)
	}

	fn disabled_color(&self, style: &Self::Style) -> Color {
		self.placeholder_color(style)
	}
}