use crate::styles::mix_color;
use iced::{Color, Theme};
use iced_aw::{card::Status, style::card::Style};

pub fn card_style(theme: &Theme, _status: Status) -> Style {
	let background_color = card_background_color(theme);

	Style {
		border_width: 0.0,
		border_radius: 15.0,
		background: background_color.into(),
		body_text_color: theme.extended_palette().background.base.text,
		head_background: background_color.into(),
		head_text_color: theme.extended_palette().background.base.text,
		close_color: theme.extended_palette().background.base.text,
		..Default::default()
	}
}

pub fn card_background_color(theme: &Theme) -> Color {
	mix_color(
		theme.extended_palette().background.base.color,
		theme.extended_palette().background.weak.color,
		0.15,
	)
}
