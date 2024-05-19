use iced::{widget::checkbox::{Appearance, StyleSheet}, Border, Color, Theme};

pub struct GreenCheckboxStyle;

impl StyleSheet for GreenCheckboxStyle {
	type Style = Theme;
	
	fn active(&self, style: &Self::Style, is_checked: bool) -> Appearance {
		Appearance {
			background: iced::Background::Color(
				if is_checked {
					Color::from_rgb(0.0, 1.0, 0.0)
				}
				else {
					style.extended_palette().background.base.color
				}
			),
			icon_color: style.extended_palette().success.base.text,
			text_color: None,
			border: Border {
				radius: 2.0.into(),
				width: 1.0,
				color: Color::from_rgb(0.0, 1.0, 0.0),
			},
		}
	}

	fn hovered(&self, style: &Self::Style, is_checked: bool) -> Appearance {
		Appearance {
			background: iced::Background::Color(
				if is_checked {
					Color::from_rgb(0.0, 1.0, 0.0)
				}
				else {
					style.extended_palette().background.weak.color
				}
			),
			icon_color: style.extended_palette().primary.strong.text,
			text_color: None,
			border: Border {
				radius: 2.0.into(),
				width: 1.0,
				color: Color::from_rgb(0.0, 1.0, 0.0),
			},
		}
	}
}