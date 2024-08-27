use iced::{widget::checkbox::{Appearance, StyleSheet}, Border, Theme};
use crate::styles::color_average;

pub struct GreenCheckboxStyle;

impl StyleSheet for GreenCheckboxStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style, is_checked: bool) -> Appearance {
		Appearance {
			background:
				if is_checked {
					style.extended_palette().primary.base.color.into()
				}
				else {
					style.extended_palette().background.base.color.into()
				},
			icon_color: style.extended_palette().success.base.text,
			text_color: None,
			border: Border {
				radius: 2.0.into(),
				width: 1.0,
				color: if is_checked {
					style.extended_palette().primary.base.color
				}
				else {
					style.extended_palette().background.weak.color
				},
			},
		}
	}

	fn hovered(&self, style: &Self::Style, is_checked: bool) -> Appearance {
		Appearance {
			background:
				if is_checked {
					color_average(
						style.extended_palette().primary.base.color,
						style.extended_palette().background.base.text
					)
					.into()
				}
				else {
					style.extended_palette().background.weak.color.into()
				},
			icon_color: style.extended_palette().success.base.text,
			text_color: None,
			border: Border {
				radius: 2.0.into(),
				width: 1.0,
				color: if is_checked {
					color_average(
						style.extended_palette().primary.base.color,
						style.extended_palette().background.base.text
					)
				}
				else {
					style.extended_palette().background.strong.color
				},
			},
		}
	}
}