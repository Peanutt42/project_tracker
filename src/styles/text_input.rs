use crate::styles::{BORDER_RADIUS, selection_color};
use iced::{
	border::Radius,
	widget::text_input::{Status, Style},
	Border, Theme,
};

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

pub fn text_input_style_borderless(theme: &Theme, status: Status) -> Style {
	let placeholder = theme.extended_palette().background.strong.color;
	let value = theme.extended_palette().background.base.text;
	let selection = selection_color(theme.extended_palette());

	let border = Border::default();

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
