use crate::styles::{selection_color, BORDER_RADIUS};
use std::str::FromStr;
use iced::{
	border::Radius,
	widget::text_input::{Status, Style},
	Border, Theme,
};

use super::card_style::card_background_color;

pub fn text_input_style_default(theme: &Theme, status: Status) -> Style {
	text_input_style(theme, status, true, true, true, true)
}

pub fn text_input_style_only_round_left(theme: &Theme, status: Status) -> Style {
	text_input_style(theme, status, true, false, false, true)
}

pub fn text_input_style(
	theme: &Theme,
	status: Status,
	round_left_top: bool,
	round_right_top: bool,
	round_right_bottom: bool,
	round_left_bottom: bool,
) -> Style {
	let placeholder = theme.extended_palette().background.strong.color;
	let value = theme.extended_palette().background.base.text;
	let selection = selection_color(theme.extended_palette());

	let border = Border {
		radius: Radius::default()
			.top_left(if round_left_top { BORDER_RADIUS } else { 0.0 })
			.top_right(if round_right_top { BORDER_RADIUS } else { 0.0 })
			.bottom_left(if round_left_bottom {
				BORDER_RADIUS
			} else {
				0.0
			})
			.bottom_right(if round_right_bottom {
				BORDER_RADIUS
			} else {
				0.0
			}),
		width: 1.0,
		color: theme.extended_palette().background.strong.color,
	};

	match status {
		Status::Active | Status::Hovered | Status::Focused => Style {
			background: theme.extended_palette().background.base.color.into(),
			border,
			icon: theme.extended_palette().background.weak.text,
			placeholder,
			value,
			selection,
		},
		Status::Disabled => Style {
			background: theme.extended_palette().background.weak.color.into(),
			icon: theme.extended_palette().background.strong.color,
			border,
			placeholder,
			value,
			selection,
		},
	}
}

pub fn text_input_style_borderless(theme: &Theme, status: Status, inside_card: bool) -> Style {
	let placeholder = theme.extended_palette().background.strong.color;
	let value = theme.extended_palette().background.base.text;
	let selection = selection_color(theme.extended_palette());

	let border = Border::default();

	match status {
		Status::Active | Status::Hovered | Status::Focused => Style {
			background: if inside_card {
				card_background_color(theme)
			} else {
				theme.extended_palette().background.base.color
			}.into(),
			border,
			icon: theme.extended_palette().background.weak.text,
			placeholder,
			value,
			selection,
		},
		Status::Disabled => Style {
			background: theme.extended_palette().background.weak.color.into(),
			icon: theme.extended_palette().background.strong.color,
			border,
			placeholder,
			value,
			selection,
		},
	}
}

pub fn on_number_input<Message>(input: String, on_number: impl FnOnce(Option<usize>) -> Message, on_invalid_input: Message) -> Message {
	let new_number = match usize::from_str(&input) {
		Ok(new_number) => {
			Some(Some(new_number))
		}
		Err(_) => {
			if input.is_empty() {
				Some(None)
			} else {
				None
			}
		}
	};
	match new_number {
		Some(new_number) => on_number(new_number),
		None => on_invalid_input,
	}
}