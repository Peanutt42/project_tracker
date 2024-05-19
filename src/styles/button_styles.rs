use iced::{widget::button, Background, Border, Color, Theme};

pub struct ProjectPreviewButtonStyle {
	pub selected: bool,
}

impl button::StyleSheet for ProjectPreviewButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(
				if self.selected {
					style.extended_palette().secondary.base.color
				}
				else {
					style.palette().background
				}
			)),
			text_color: style.palette().text,
			border: Border::with_radius(5.0),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(style.extended_palette().secondary.base.color)),
			text_color: style.palette().text,
			border: Border::with_radius(5.0),
			..Default::default()
		}
	}

	fn pressed(&self, style: &Self::Style) -> button::Appearance {
		self.hovered(style)
	}
}

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
			border: Border::with_radius(f32::MAX),
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 1.0, 0.0))),
			border: Border::with_radius(f32::MAX),
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> button::Appearance {
		button::Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.8, 0.0))),
			border: Border::with_radius(f32::MAX),
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
