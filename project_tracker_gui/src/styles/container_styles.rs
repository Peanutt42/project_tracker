use iced::{widget::container::Style, Border, border::rounded, Color, Shadow, Theme, Vector};
use crate::styles::{BLUR_RADIUS, BORDER_RADIUS, LARGE_BORDER_RADIUS, SELECTION_COLOR, mix_color, background_shadow_color, color_average};

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
                a: if theme.extended_palette().is_dark { 0.5 } else { 1.0 },
                ..background_shadow_color(theme.extended_palette())
            },
            blur_radius: BLUR_RADIUS,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn palette_container_style(theme: &Theme) -> Style {
    Style {
        background: Some(
            mix_color(theme.extended_palette().background.weak.color, theme.extended_palette().background.base.color, 0.25).into(),
        ),
        border: rounded(LARGE_BORDER_RADIUS),
        shadow: Shadow {
            blur_radius: if theme.extended_palette().is_dark { 50.0 } else { 35.0 },
            color: background_shadow_color(theme.extended_palette()),
            offset: Vector::ZERO,
        },
        ..Default::default()
    }
}

pub fn dropzone_container_style(_theme: &Theme, highlight: bool) -> Style {
    Style {
        background: None,
        border: Border {
            color: SELECTION_COLOR,
            width: if highlight { 2.0 } else { 0.0 },
            radius: BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

pub fn in_between_dropzone_container_style(_theme: &Theme, highlight: bool) -> Style {
    Style {
        background: if highlight {
            Some(SELECTION_COLOR.into())
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
            Some(color_average(theme.extended_palette().background.weak.color, theme.extended_palette().background.base.color).into())
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

pub fn shadow_container_style(theme: &Theme) -> Style {
    Style {
        border: rounded(BORDER_RADIUS),
        shadow: Shadow {
            color: background_shadow_color(theme.extended_palette()),
            blur_radius: BLUR_RADIUS,
            ..Default::default()
        },
        ..Default::default()
    }
}
