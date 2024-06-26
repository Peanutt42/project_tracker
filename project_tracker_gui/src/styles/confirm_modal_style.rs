use iced::{Color, Theme};
use iced_aw::{card, modal};
use iced_aw::style;

pub struct ConfirmModalStyle;

impl modal::StyleSheet for ConfirmModalStyle {
	type Style = Theme;

	fn active(&self, _style: &Self::Style) -> style::modal::Appearance {
		style::modal::Appearance {
			background: Color::from_rgba(0.0, 0.0, 0.0, 0.75).into()
		}
	}
}

pub struct ConfirmModalCardStyle;

impl card::StyleSheet for ConfirmModalCardStyle {
	type Style = Theme;

	fn active(&self, style: &Self::Style) -> style::card::Appearance {
		card::Appearance {
			border_width: 0.0,
			background: style.extended_palette().background.base.color.into(),
			head_background: style.extended_palette().background.base.color.into(),
			head_text_color: style.extended_palette().background.base.text,
			..Default::default()
		}
	}
}