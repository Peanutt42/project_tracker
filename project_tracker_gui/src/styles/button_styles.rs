use iced::{color, widget::button::{Appearance, StyleSheet}, Border, Color, Shadow, Theme};
use crate::styles::{BORDER_RADIUS, LARGE_BORDER_RADIUS, SMALL_BLUR_RADIUS, BLUR_RADIUS, color_average, mix_color, background_shadow_color, text_color};

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
					mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color, 0.75).into()
				}
			),
			text_color: style.extended_palette().background.base.text,
			border: Border {
				radius: BORDER_RADIUS.into(),
				color: color!(0x3584e4),
				width: if self.selected { 2.0 } else { 0.0 },
			},
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
					color_average(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()
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
			background: Some(color_average(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()),
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

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(1.0, 0.0, 0.0).into()),
			text_color: style.extended_palette().danger.base.text,
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		let color = Color::from_rgb(0.8, 0.0, 0.0);

		Appearance {
			background: Some(color.into()),
			shadow: Shadow {
				color: Color { a: if style.extended_palette().is_dark { 0.25 } else { 1.0 }, ..color },
				blur_radius: SMALL_BLUR_RADIUS,
				..Default::default()
			},
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.6, 0.0, 0.0).into()),
			..self.active(style)
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

	fn hovered(&self, style: &Self::Style) -> Appearance {
		let color = Color::from_rgb(0.8, 0.0, 0.0);

		Appearance {
			background: Some(color.into()),
			text_color: style.extended_palette().danger.base.text,
			shadow: Shadow {
				color: Color { a: 0.25, ..color },
				blur_radius: SMALL_BLUR_RADIUS,
				..Default::default()
			},
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(Color::from_rgb(0.6, 0.0, 0.0).into()),
			..self.hovered(style)
		}
	}
}

pub struct SecondaryButtonStyle {
	pub round_left_top: bool,
	pub round_left_bottom: bool,
	pub round_right_top: bool,
	pub round_right_bottom: bool,
}

impl SecondaryButtonStyle {
	pub const ONLY_ROUND_RIGHT: Self = Self {
		round_left_top: false,
		round_right_top: true,
		round_right_bottom: true,
		round_left_bottom: false,
	};

	pub const ONLY_ROUND_BOTTOM: Self = Self {
		round_left_top: false,
		round_left_bottom: true,
		round_right_top: false,
		round_right_bottom: true,
	};

	pub const NO_ROUNDING: Self = Self {
		round_left_top: false,
		round_left_bottom: false,
		round_right_top: false,
		round_right_bottom: false,
	};
}

impl Default for SecondaryButtonStyle {
	fn default() -> Self {
		Self {
			round_left_top: true,
			round_left_bottom: true,
			round_right_top: true,
			round_right_bottom: true,
		}
	}
}

impl StyleSheet for SecondaryButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		let pair = style.extended_palette().secondary.base;

		Appearance {
			background: Some(pair.color.into()),
			text_color: pair.text,
			border: Border::with_radius([
				if self.round_left_top { BORDER_RADIUS } else { 0.0 },
				if self.round_right_top { BORDER_RADIUS } else { 0.0 },
				if self.round_right_bottom { BORDER_RADIUS } else { 0.0 },
				if self.round_left_bottom { BORDER_RADIUS } else { 0.0 }
			]),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().background.strong.color.into()),
			text_color: style.extended_palette().secondary.base.text,
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.active(style)
	}
}

pub struct DeleteButtonStyle {
	pub round_left: bool,
	pub round_right: bool,
}

impl StyleSheet for DeleteButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().secondary.base.color.into()),
			text_color: style.extended_palette().secondary.base.text,
			border: Border::with_radius([
				if self.round_left { BORDER_RADIUS } else { 0.0 },
				if self.round_right { BORDER_RADIUS } else { 0.0 },
				if self.round_right { BORDER_RADIUS } else { 0.0 },
				if self.round_left { BORDER_RADIUS } else { 0.0 }
			]),
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().danger.base.color.into()),
			shadow: Shadow {
				color: Color {
					a: if style.extended_palette().is_dark { 0.25 } else { 1.0 },
					..style.extended_palette().danger.base.color
				},
				blur_radius: SMALL_BLUR_RADIUS,
				..Default::default()
			},
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(color_average(style.extended_palette().background.base.color, style.extended_palette().danger.weak.color).into()),
			..self.active(style)
		}
	}
}

pub struct SelectionListButtonStyle {
	pub selected: bool,
	pub round_left: bool,
	pub round_right: bool,
}

impl StyleSheet for SelectionListButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(
				if self.selected {
					style.extended_palette().primary.base.color.into()
				}
				else {
					style.extended_palette().secondary.base.color.into()
				}
			),
			border: Border::with_radius([
				if self.round_left { BORDER_RADIUS } else { 0.0 },
				if self.round_right { BORDER_RADIUS } else { 0.0 },
				if self.round_right { BORDER_RADIUS } else { 0.0 },
				if self.round_left { BORDER_RADIUS } else { 0.0 },
			]),
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
					style.extended_palette().success.base.color.into()
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
					color_average(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()
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
				color_average(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()
			}),
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.hovered(style)
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

pub struct TaskTagButtonStyle {
	pub color: Color,
	pub toggled: bool,
	pub round_bottom: bool,
}

impl StyleSheet for TaskTagButtonStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(if self.toggled {
				self.color.into()
			}
			else {
				style.extended_palette().background.base.color.into()
			}),
			text_color: if self.toggled {
				text_color(self.color)
			}
			else {
				style.extended_palette().background.base.text
			},
			border: Border {
				color: self.color,
				width: 1.0,
				radius: if self.round_bottom {
					LARGE_BORDER_RADIUS.into()
				}
				else {
					[
						LARGE_BORDER_RADIUS,
						LARGE_BORDER_RADIUS,
						0.0,
						0.0
					].into()
				},
			},
			..Default::default()
		}
	}

	fn hovered(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(
				if self.toggled {
					self.color.into()
				}
				else {
					color_average(self.color, style.extended_palette().background.base.color).into()
				}
			),
			shadow: Shadow {
				color: Color {
					a: if style.extended_palette().is_dark { 0.25 } else { 1.0 },
					..self.color
				},
				blur_radius: SMALL_BLUR_RADIUS,
				..Default::default()
			},
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(self.color.into()),
			..self.active(style)
		}
	}
}

pub struct TaskButtonStyle;

impl StyleSheet for TaskButtonStyle {
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
			background: Some(color_average(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into()),
			shadow: Shadow {
				color: background_shadow_color(style.extended_palette()),
				blur_radius: BLUR_RADIUS,
				..Default::default()
			},
			..self.active(style)
		}
	}

	fn pressed(&self, style: &Self::Style) -> Appearance {
		self.hovered(style)
	}
}