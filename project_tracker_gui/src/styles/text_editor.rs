use iced::{border::Radius, color, widget::text_editor::{Status, Style}, Border, Theme};
use crate::styles::BORDER_RADIUS;

pub fn text_editor_style(theme: &Theme, status: Status, round_top_left: bool, round_top_right: bool, round_bottom_left: bool, round_bottom_right: bool) -> Style {
	let placeholder = theme.extended_palette().background.strong.color;
	let value = theme.extended_palette().background.base.text;
	let selection = color!(0x3367d1);

	let border = Border {
		radius: Radius::default()
			.top_left(if round_top_left { BORDER_RADIUS } else { 0.0 })
			.top_right(if round_top_right { BORDER_RADIUS } else { 0.0 })
			.bottom_left(if round_bottom_left { BORDER_RADIUS } else { 0.0 })
			.bottom_right(if round_bottom_right { BORDER_RADIUS } else { 0.0 }),
		width: 1.0,
		color: theme.extended_palette().background.strong.color,
	};

	match status {
		Status::Active | Status::Hovered | Status::Focused => {
			Style {
				background: theme.extended_palette().background.base.color.into(),
				icon: theme.extended_palette().background.weak.text,
				border,
				placeholder,
				value,
				selection
			}
		},
		Status::Disabled => {
			Style {
				background: theme.extended_palette().background.weak.color.into(),
				icon: theme.extended_palette().background.strong.text,
				border,
				placeholder,
				value,
				selection
			}
		},
	}
}