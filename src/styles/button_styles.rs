use iced::{widget::button::{Appearance, StyleSheet}, Background, Border, Color, Theme};

use crate::styles::{NICE_GREEN, DARK_GREEN};

pub struct ProjectPreviewButtonStyle {
	pub selected: bool,
}

impl StyleSheet for ProjectPreviewButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					style.extended_palette().primary.weak.color
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

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(
				if self.selected {
					style.extended_palette().primary.weak.color
				}
				else {
					style.extended_palette().background.weak.color
				}
			)),
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.hovered(style)
	}
}

pub struct GreenButtonStyle;

impl StyleSheet for GreenButtonStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.9, 0.0))),
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 1.0, 0.0))),
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(Color::from_rgb(0.0, 0.8, 0.0))),
			border: Border::with_radius(7.5),
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
			border: Border::with_radius(5.0),
			..self.active(style)
		}
	}
}


pub struct SecondaryButtonStyle;

impl StyleSheet for SecondaryButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let pair = style.extended_palette().secondary.base;

		Appearance {
			background: Some(Background::Color(pair.color)),
			text_color: pair.text,
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Background::Color(style.extended_palette().background.strong.color)),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(7.5),
			..Default::default()
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.active(style)
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
					DARK_GREEN
				}
				else {
					style.extended_palette().background.weak.color
				}
			)),
			text_color: if self.selected {
				NICE_GREEN
			}
			else {
				style.extended_palette().background.base.text
			},
			border: Border::with_radius(f32::MAX),
			..Default::default()
		}
	}
}