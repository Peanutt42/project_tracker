use iced::{color, widget::container::{Appearance, StyleSheet}, Border, Color, Shadow, Theme, Vector};
use crate::styles::{BLUR_RADIUS, BORDER_RADIUS, LARGE_BORDER_RADIUS, mix_color, background_shadow_color, color_average};

pub struct RoundedContainerStyle;

impl StyleSheet for RoundedContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().background.weak.color.into()),
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}

pub struct TooltipContainerStyle;

impl StyleSheet for TooltipContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: Some(style.extended_palette().background.weak.color.into()),
			border: Border::with_radius(BORDER_RADIUS),
			shadow: Shadow {
				color: Color {
					a: if style.extended_palette().is_dark { 0.5 } else { 1.0 },
					..background_shadow_color(style.extended_palette())
				},
				blur_radius: BLUR_RADIUS,
				..Default::default()
			},
			..Default::default()
		}
	}
}

pub struct PaletteContainerStyle;

impl StyleSheet for PaletteContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			// 25% weak color, 75% base color
			background: Some(
				mix_color(style.extended_palette().background.weak.color, style.extended_palette().background.base.color, 0.25).into()
			),
			border: Border::with_radius(LARGE_BORDER_RADIUS),
			shadow: Shadow {
				blur_radius: if style.extended_palette().is_dark { 50.0 } else { 35.0 },
				color: background_shadow_color(style.extended_palette()),
				offset: Vector::ZERO,
			},
			..Default::default()
		}
	}
}

pub struct DropzoneContainerStyle {
	pub highlight: bool,
}

impl StyleSheet for DropzoneContainerStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: None,
			border: Border {
				color: color!(0x3584e4),
				width: if self.highlight { 2.0 } else { 0.0 },
				radius: BORDER_RADIUS.into(),
			},
			..Default::default()
		}
	}
}

pub struct InBetweenDropzoneContainerStyle {
	pub highlight: bool
}

impl StyleSheet for InBetweenDropzoneContainerStyle {
	type Style = Theme;

	fn appearance(&self, _style: &Self::Style) -> Appearance {
		Appearance {
			background: if self.highlight {
				Some(color!(0x3584e4).into())
			}
			else {
				None
			},
			..Default::default()
		}
	}
}

pub struct ProjectPreviewBackgroundContainerStyle {
	pub dragging: bool,
}

impl StyleSheet for ProjectPreviewBackgroundContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: if self.dragging {
				Some(style.extended_palette().background.weak.color.into())
			}
			else {
				None
			},
			border: Border::with_radius(BORDER_RADIUS),
			..Default::default()
		}
	}
}

pub struct TaskBackgroundContainerStyle {
	pub dragging: bool,
}

impl StyleSheet for TaskBackgroundContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			background: if self.dragging {
				Some(color_average(style.extended_palette().background.weak.color, style.extended_palette().background.base.color).into())
			}
			else {
				None
			},
			border: Border::with_radius(BORDER_RADIUS),
			shadow: if self.dragging {
				Shadow {
					color: background_shadow_color(style.extended_palette()),
					blur_radius: BLUR_RADIUS,
					..Default::default()
				}
			}
			else {
				Shadow::default()
			},
			..Default::default()
		}
	}
}