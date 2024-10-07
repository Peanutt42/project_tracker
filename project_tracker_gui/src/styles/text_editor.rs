use crate::styles::{selection_color, BORDER_RADIUS};
use iced::{
	border::Radius, keyboard::{self, key}, widget::text_editor::{Binding, KeyPress, Motion, Status, Style}, Border, Theme
};

pub fn text_editor_style(
	theme: &Theme,
	status: Status,
	round_top_left: bool,
	round_top_right: bool,
	round_bottom_left: bool,
	round_bottom_right: bool,
) -> Style {
	let placeholder = theme.extended_palette().background.strong.color;
	let value = theme.extended_palette().background.base.text;
	let selection = selection_color(theme.extended_palette());

	let border = Border {
		radius: Radius::default()
			.top_left(if round_top_left { BORDER_RADIUS } else { 0.0 })
			.top_right(if round_top_right { BORDER_RADIUS } else { 0.0 })
			.bottom_left(if round_bottom_left {
				BORDER_RADIUS
			} else {
				0.0
			})
			.bottom_right(if round_bottom_right {
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
			icon: theme.extended_palette().background.weak.text,
			border,
			placeholder,
			value,
			selection,
		},
		Status::Disabled => Style {
			background: theme.extended_palette().background.weak.color.into(),
			icon: theme.extended_palette().background.strong.text,
			border,
			placeholder,
			value,
			selection,
		},
	}
}


pub fn text_editor_keybindings<Message>(key_press: KeyPress) -> Option<Binding<Message>> {
	let KeyPress {
		key,
		modifiers,
		status,
		..
	} = &key_press;

	if *status != Status::Focused {
		return None;
	}

	match key {
		keyboard::Key::Named(key::Named::Delete) => Some(
			if modifiers.command() {
				Binding::Sequence(vec![
					Binding::Select(Motion::WordRight),
					Binding::Delete,
				])
			}
			else {
				Binding::Delete
			}
		),
		keyboard::Key::Named(key::Named::Backspace) if modifiers.command() => Some(
			Binding::Sequence(vec![
				Binding::<Message>::Select(Motion::WordLeft),
				Binding::Backspace,
			])
		),
		_ => Binding::<Message>::from_key_press(key_press)
	}
}