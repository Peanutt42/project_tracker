use crate::styles::{
	background_shadow_color, color_average, mix_color, text_color, BLUR_RADIUS, BORDER_RADIUS,
	LARGE_BORDER_RADIUS,
};
use iced::{border::rounded, widget::container::Style, Border, Color, Shadow, Theme, Vector};

use super::{link_color, LARGE_BLUR_RADIUS};

pub fn rounded_container_style(theme: &Theme) -> Style {
	Style {
		background: Some(theme.extended_palette().background.weak.color.into()),
		border: rounded(BORDER_RADIUS),
		..Default::default()
	}
}

pub fn tooltip_container_style(theme: &Theme) -> Style {
	Style {
		background: Some(theme.extended_palette().background.weak.color.into()),
		border: rounded(BORDER_RADIUS),
		shadow: Shadow {
			color: Color {
				a: if theme.extended_palette().is_dark {
					0.5
				} else {
					1.0
				},
				..background_shadow_color(theme.extended_palette())
			},
			blur_radius: BLUR_RADIUS,
			..Default::default()
		},
		..Default::default()
	}
}

pub fn dropdown_container_style(theme: &Theme) -> Style {
	Style {
		background: None,
		border: rounded(BORDER_RADIUS),
		shadow: Shadow {
			color: Color {
				a: if theme.extended_palette().is_dark {
					0.5
				} else {
					1.0
				},
				..background_shadow_color(theme.extended_palette())
			},
			blur_radius: LARGE_BLUR_RADIUS,
			..Default::default()
		},
		..Default::default()
	}
}

pub fn palette_container_style(theme: &Theme) -> Style {
	Style {
		background: Some(
			mix_color(
				theme.extended_palette().background.weak.color,
				theme.extended_palette().background.base.color,
				0.25,
			)
			.into(),
		),
		border: rounded(LARGE_BORDER_RADIUS),
		shadow: Shadow {
			blur_radius: if theme.extended_palette().is_dark {
				50.0
			} else {
				35.0
			},
			color: background_shadow_color(theme.extended_palette()),
			offset: Vector::ZERO,
		},
		..Default::default()
	}
}

pub fn dropzone_container_style(theme: &Theme, highlight: bool) -> Style {
	Style {
		background: None,
		border: Border {
			color: link_color(theme.extended_palette().is_dark),
			width: if highlight { 3.5 } else { 0.0 },
			radius: BORDER_RADIUS.into(),
		},
		..Default::default()
	}
}

pub fn in_between_dropzone_container_style(theme: &Theme, highlight: bool) -> Style {
	Style {
		background: if highlight {
			Some(link_color(theme.extended_palette().is_dark).into())
		} else {
			None
		},
		..Default::default()
	}
}

pub fn project_preview_background_container_style(theme: &Theme, dragging: bool) -> Style {
	Style {
		background: if dragging {
			Some(theme.extended_palette().background.weak.color.into())
		} else {
			None
		},
		border: rounded(BORDER_RADIUS),
		..Default::default()
	}
}

pub fn task_background_container_style(theme: &Theme, dragging: bool) -> Style {
	Style {
		background: if dragging {
			Some(
				color_average(
					theme.extended_palette().background.weak.color,
					theme.extended_palette().background.base.color,
				)
				.into(),
			)
		} else {
			None
		},
		border: rounded(BORDER_RADIUS),
		shadow: if dragging {
			Shadow {
				color: background_shadow_color(theme.extended_palette()),
				blur_radius: BLUR_RADIUS,
				..Default::default()
			}
		} else {
			Shadow::default()
		},
		..Default::default()
	}
}

pub fn markdown_background_container_style(theme: &Theme) -> Style {
	Style {
		background: Some(mix_color(theme.extended_palette().background.base.color, theme.extended_palette().background.strong.color, 0.25).into()),
		border: rounded(BORDER_RADIUS),
		..Default::default()
	}
}

pub fn task_tag_container_style(_theme: &Theme, color: Color) -> Style {
	Style {
		background: Some(color.into()),
		text_color: Some(text_color(color)),
		border: Border {
			color,
			width: 1.0,
			radius: LARGE_BORDER_RADIUS.into(),
		},
		..Default::default()
	}
}
