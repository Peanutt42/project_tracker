use iced::{widget::button::{Appearance, StyleSheet}, Background, Border, Color, Theme};
use crate::styles::{mix_color, NICE_GREEN, LIGHT_DARK_GREEN, BORDER_RADIUS, CIRCLE_BORDER_RADIUS, LARGE_BORDER_RADIUS};

pub struct ProjectPreviewButtonStyle {
	pub selected: bool,
}

impl StyleSheet for ProjectPreviewButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					style.extended_palette().background.weak.color
				}
				else {
					style.extended_palette().background.base.color
				}
			)),
			text_color: style.palette().text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					style.extended_palette().background.weak.color
				}
				else {
					mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color)
				}
			)),
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.hovered(style)
	}
}

pub struct DangerousButtonStyle;

impl StyleSheet for DangerousButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(1.0, 0.0, 0.0))),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.8, 0.0, 0.0))),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.6, 0.0, 0.0))),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}


pub struct TransparentButtonStyle;

impl StyleSheet for TransparentButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::TRANSPARENT)),
			text_color: style.extended_palette().secondary.base.text,
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(style.extended_palette().background.strong.color)),
			border: Border::with_radius(BORDER_RADIUS),
			..self.active(style)
		}
	}
}


pub struct ProjectContextButtonStyle;

impl StyleSheet for ProjectContextButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let pair = style.extended_palette().secondary.base;

		Appearance {
			background: Some(Background::Color(pair.color)),
			text_color: pair.text,
			border: Border::with_radius(LARGE_BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(style.extended_palette().background.strong.color)),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(LARGE_BORDER_RADIUS),
			..Default::default()
		}
	}
}

pub struct DeleteButtonStyle;

impl StyleSheet for DeleteButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(1.0, 0.0, 0.0))),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.55, 0.0, 0.0))),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.45, 0.0, 0.0))),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}

pub struct InvisibleButtonStyle;

impl StyleSheet for InvisibleButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: None,
			text_color: Color::TRANSPARENT,
			..Default::default()
		}
	}
}

pub struct TaskFilterButtonStyle {
	pub selected: bool,
}

impl StyleSheet for TaskFilterButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					LIGHT_DARK_GREEN
				}
				else {
					style.extended_palette().secondary.base.color
				}
			)),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(CIRCLE_BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					NICE_GREEN
				}
				else {
					style.extended_palette().background.strong.color
				}
			)),
			..self.active(style)
		}
	}
}

pub struct ThemeModeButtonStyle {
	pub selected: bool,
}

impl StyleSheet for ThemeModeButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					NICE_GREEN
				}
				else {
					style.extended_palette().secondary.base.color
				}
			)),
			border: Border::with_radius(BORDER_RADIUS),
			text_color: if self.selected {
				style.extended_palette().primary.base.text
			}
			else {
				style.extended_palette().secondary.base.text
			},
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					LIGHT_DARK_GREEN
				}
				else {
					style.extended_palette().background.strong.color
				}
			)),
			..self.active(style)
		}
	}
}
