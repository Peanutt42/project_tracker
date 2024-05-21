use iced::{widget::text_input::{Appearance, StyleSheet}, Color, Border, Theme};


pub struct TextInputStyle;

impl StyleSheet for TextInputStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

		Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
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
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.base.text,
            },
            icon_color: palette.background.weak.text,
        }
	}

	fn focused(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

        Appearance {
            background: palette.background.base.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: Color::from_rgb(0.0, 1.0, 0.0),//palette.primary.strong.color,
            },
            icon_color: palette.background.weak.text,
        }
	}

	fn disabled(&self, style: &Self::Style) -> Appearance {
		let palette = style.extended_palette();

        Appearance {
            background: palette.background.weak.color.into(),
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: palette.background.strong.color,
            },
            icon_color: palette.background.strong.color,
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