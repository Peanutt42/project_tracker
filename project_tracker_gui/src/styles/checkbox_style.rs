use iced::{widget::checkbox::{Style, Status}, Border, Theme};
use crate::styles::color_average;

pub fn checkbox_style(theme: &Theme, status: Status) -> Style {
	match status {
		Status::Active { is_checked } => {
			Style {
				background:
					if is_checked {
						theme.extended_palette().primary.base.color.into()
					}
					else {
						theme.extended_palette().background.base.color.into()
					},
				icon_color: theme.extended_palette().success.base.text,
				text_color: None,
				border: Border {
					radius: 2.0.into(),
					width: 1.0,
					color: if is_checked {
						theme.extended_palette().primary.base.color
					}
					else {
						theme.extended_palette().background.weak.color
					},
				},
			}
		},
		Status::Hovered { is_checked } | Status::Disabled { is_checked } => {
			Style {
				background:
					if is_checked {
						color_average(
							theme.extended_palette().primary.base.color,
							theme.extended_palette().background.base.text
						)
						.into()
					}
					else {
						theme.extended_palette().background.weak.color.into()
					},
				icon_color: theme.extended_palette().success.base.text,
				text_color: None,
				border: Border {
					radius: 2.0.into(),
					width: 1.0,
					color: if is_checked {
						color_average(
							theme.extended_palette().primary.base.color,
							theme.extended_palette().background.base.text
						)
					}
					else {
						theme.extended_palette().background.strong.color
					},
				},
			}
		},
	}
}