use iced::{widget::button::{Appearance, StyleSheet}, Border, Color, Theme, Vector};
use crate::styles::{mix_color, NICE_GREEN, LIGHT_DARK_GREEN, BORDER_RADIUS, LARGE_BORDER_RADIUS};

pub struct ProjectPreviewButtonStyle {
	pub selected: bool,
}

impl StyleSheet for ProjectPreviewButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(
				if self.selected {
					style.extended_palette().background.weak.color.into()
				}
				else {
					style.extended_palette().background.base.color.into()
				}
			),
			text_color: style.palette().text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(
				if self.selected {
					style.extended_palette().background.weak.color.into()
				}
				else {
					mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()
				}
			),
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.hovered(style)
	}
}

pub struct HiddenSecondaryButtonStyle;

impl StyleSheet for HiddenSecondaryButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: None,
			text_color: style.palette().text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()),
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
			background: Some(Color::from_rgb(1.0, 0.0, 0.0).into()),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.8, 0.0, 0.0).into()),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.6, 0.0, 0.0).into()),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}

pub struct DeleteDoneTasksButtonStyle;

impl StyleSheet for DeleteDoneTasksButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().secondary.base.color.into()),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.8, 0.0, 0.0).into()),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn pressed(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.6, 0.0, 0.0).into()),
			text_color: Color::WHITE,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}


pub struct RoundedSecondaryButtonStyle;

impl StyleSheet for RoundedSecondaryButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			border: Border::with_radius(BORDER_RADIUS),
			background: Some(style.extended_palette().secondary.base.color.into()),
			text_color: style.extended_palette().secondary.base.text,
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().background.strong.color.into()),
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			shadow_offset: Vector::default(),
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
			background: Some(pair.color.into()),
			text_color: pair.text,
			border: Border::with_radius(LARGE_BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().background.strong.color.into()),
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
			background: Some(Color::from_rgb(1.0, 0.0, 0.0).into()),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.55, 0.0, 0.0).into()),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.45, 0.0, 0.0).into()),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
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
			background: Some(
				if self.selected {
					NICE_GREEN.into()
				}
				else {
					style.extended_palette().secondary.base.color.into()
				}
			),
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
			background: Some(
				if self.selected {
					LIGHT_DARK_GREEN.into()
				}
				else {
					style.extended_palette().background.strong.color.into()
				}
			),
			..self.active(style)
		}
	}
}


pub struct PaletteItemButtonStyle {
	pub selected: bool,
}

impl StyleSheet for PaletteItemButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: if self.selected {
				Some(style.extended_palette().background.weak.color.into())
			}
			else {
				None
			},
			text_color: style.extended_palette().background.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(
				if self.selected {
					style.extended_palette().background.weak.color.into()
				}
				else {
					mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()
				}
			),
			..self.active(style)
		}
	}
}


pub struct ColorPaletteButtonStyle {
	pub selected: bool,
}

impl StyleSheet for ColorPaletteButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: if self.selected {
				Some(style.extended_palette().background.weak.color.into())
		 	}
			else {
				None
			},
			text_color: style.palette().text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(if self.selected {
				style.extended_palette().background.weak.color.into()
			}
			else {
				mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()
			}),
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.hovered(style)
	}
}