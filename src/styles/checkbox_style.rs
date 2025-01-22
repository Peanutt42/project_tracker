use crate::styles::color_average;
use iced::{
	widget::checkbox::{Status, Style},
	Border, Theme,
};
use iced_aw::style::colors::WHITE;

pub fn checkbox_style(theme: &Theme, status: Status) -> Style {
	match status {
		Status::Active { is_checked } => Style {
			background: if is_checked {
				theme.extended_palette().primary.strong.color.into()
			} else {
				theme.extended_palette().background.base.color.into()
			},
			icon_color: WHITE,
			text_color: None,
			border: Border {
				radius: 2.5.into(),
				width: 1.0,
				color: if is_checked {
					theme.extended_palette().primary.strong.color
				} else {
					theme.extended_palette().background.weak.color
				},
			},
		},
		Status::Hovered { is_checked } | Status::Disabled { is_checked } => Style {
			background: if is_checked {
				color_average(
					theme.extended_palette().primary.strong.color,
					theme.extended_palette().background.base.text,
				)
				.into()
			} else {
				theme.extended_palette().background.weak.color.into()
			},
			icon_color: WHITE,
			text_color: None,
			border: Border {
				radius: 2.5.into(),
				width: 1.0,
				color: if is_checked {
					color_average(
						theme.extended_palette().primary.strong.color,
						theme.extended_palette().background.base.text,
					)
				} else {
					theme.extended_palette().background.strong.color
				},
			},
		},
	}
}
