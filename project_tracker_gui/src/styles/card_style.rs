use iced::Theme;
use iced_aw::{card::Status, style::card::Style};

pub fn card_style(theme: &Theme, _status: Status) -> Style {
	Style {
		border_width: 0.0,
		border_radius: 15.0,
		background: theme.extended_palette().background.base.color.into(),
		body_text_color: theme.extended_palette().background.base.text,
		head_background: theme.extended_palette().background.base.color.into(),
		head_text_color: theme.extended_palette().background.base.text,
		close_color: theme.extended_palette().background.base.text,
		..Default::default()
	}
}