use iced::{widget::button, Background, Border, Color, Theme, Shadow, Vector};

pub struct GreenButtonStyle;

impl button::StyleSheet for GreenButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.9, 0.0))),
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 1.0, 0.0))),
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.8, 0.0))),
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}
}



pub struct GreenCircleButtonStyle;

impl button::StyleSheet for GreenCircleButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.9, 0.0))),
			border: Border::with_radius(32.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 20.0,
			},
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 1.0, 0.0))),
			border: Border::with_radius(32.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 30.0,
			},
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.8, 0.0))),
			border: Border::with_radius(32.0),
			shadow: Shadow {
				color: Color::BLACK,
				offset: Vector::default(),
				blur_radius: 40.0,
			},
			..Default::default()
		}
	}
}


pub struct SecondaryButtonStyle;

impl button::StyleSheet for SecondaryButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> button::Appearance {
		let pair = style.extended_palette().secondary.base;

		button::Appearance {
			background: Some(Background::Color(pair.color)),
			text_color: pair.text,
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(style.extended_palette().background.strong.color)),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn pressed(&self, style: &Self::Style) -> button::Appearance {
		self.active(style)
	}
}
