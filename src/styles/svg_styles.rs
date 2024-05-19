use iced::{Color, Theme, widget::svg::{StyleSheet, Appearance}};

pub struct GreenSvgStyle;

impl StyleSheet for GreenSvgStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			color: Some(Color::from_rgb(0.0, 1.0, 0.0))
		}
	}
}

pub struct BlackWhiteSvgStyle;

impl StyleSheet for BlackWhiteSvgStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			color: Some(
				if style.extended_palette().is_dark {
					Color::WHITE
				}
				else {
					Color::BLACK
				}
			)
		}
	}
}