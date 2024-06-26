use iced::{widget::checkbox::{Appearance, StyleSheet}, Border, Theme};
use crate::styles::NICE_GREEN;

pub struct GreenCheckboxStyle;

impl StyleSheet for GreenCheckboxStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style, is_checked: bool) -> Appearance {
		Appearance {
			background:
				if is_checked {
					NICE_GREEN.into()
				}
				else {
					style.extended_palette().background.base.color.into()
				},
			icon_color: style.extended_palette().success.base.text,
			text_color: None,
			border: Border {
				radius: 2.0.into(),
				width: 1.0,
				color: NICE_GREEN,
			},
		}
	}

	fn hovered(&self, style: &Self::Style, is_checked: bool) -> Appearance {
		Appearance {
			background:
				if is_checked {
					NICE_GREEN.into()
				}
				else {
					style.extended_palette().background.weak.color.into()
				},
			icon_color: style.extended_palette().primary.strong.text,
			text_color: None,
			border: Border {
				radius: 2.0.into(),
				width: 1.0,
				color: NICE_GREEN,
			},
		}
	}
}