use iced::{widget::container::{Appearance, StyleSheet}, Border, Color, Shadow, Theme, Vector};
use crate::styles::{BLUR_RADIUS, BORDER_RADIUS, LARGE_BORDER_RADIUS, SELECTION_COLOR, mix_color, background_shadow_color, color_average};

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
				color: SELECTION_COLOR,
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
				Some(SELECTION_COLOR.into())
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

pub struct ShadowContainerStyle;

impl StyleSheet for ShadowContainerStyle {
	type Style = Theme;

	fn appearance(&self, style: &Self::Style) -> Appearance {
		Appearance {
			border: Border::with_radius(BORDER_RADIUS),
			shadow: Shadow {
				color: background_shadow_color(style.extended_palette()),
				blur_radius: BLUR_RADIUS,
				..Default::default()
			},
			..Default::default()
		}
	}
}