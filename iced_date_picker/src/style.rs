//! Status Enum of an mouse Event.
//!
/// The Status of a widget event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
	/// can be pressed.
	Active,
	/// can be pressed and it is being hovered.
	Hovered,
	/// is being pressed.
	Pressed,
	/// cannot be pressed.
	Disabled,
	/// is focused.
	Focused,
	/// is Selected.
	Selected,
}

/// The style function of widget.
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;




/// The state of the style
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StyleState {
	/// Use the active style
	Active,
	/// Use the selected style
	Selected,
	/// Use the hovered style
	Hovered,
	/// Use the focused style
	Focused,
}





use iced::{Background, Color, Theme};

/// The appearance of a [`DatePicker`](crate::widget::DatePicker).
#[derive(Clone, Copy, Debug)]
pub struct Style {
	/// The background of the [`DatePicker`](crate::widget::DatePicker).
	pub background: Background,

	/// The border radius of the [`DatePicker`](crate::widget::DatePicker).
	pub border_radius: f32,

	/// The border with of the [`DatePicker`](crate::widget::DatePicker).
	pub border_width: f32,

	/// The border color of the [`DatePicker`](crate::widget::DatePicker).
	pub border_color: Color,

	/// The text color of the [`DatePicker`](crate::widget::DatePicker).
	pub text_color: Color,

	/// The attenuated color of the days which are not in the selected month
	/// of the [`DatePicker`](crate::widget::DatePicker).
	pub text_attenuated_color: Color,

	/// The background of the days in the calender of the
	/// [`DatePicker`](crate::widget::DatePicker).
	pub day_background: Background,
}

/// The Catalog of a [`DatePicker`](crate::widget::DatePicker).
pub trait Catalog {
	///Style for the trait to use.
	type Class<'a>;

	/// The default class produced by the [`Catalog`].
	fn default<'a>() -> Self::Class<'a>;

	/// The [`Style`] of a class with the given status.
	fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
	type Class<'a> = StyleFn<'a, Self, Style>;

	fn default<'a>() -> Self::Class<'a> {
		Box::new(primary)
	}

	fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
		class(self, status)
	}
}

/// The primary theme of a [`Badge`](crate::widget::badge::Badge).
#[must_use]
pub fn primary(theme: &Theme, status: Status) -> Style {
	let palette = theme.extended_palette();
	let foreground = theme.palette();

	let base = Style {
		background: palette.background.base.color.into(),
		border_radius: 15.0,
		border_width: 1.0,
		border_color: foreground.text,
		text_color: foreground.text,
		text_attenuated_color: Color {
			a: foreground.text.a * 0.5,
			..foreground.text
		},
		day_background: palette.background.base.color.into(),
	};

	match status {
		Status::Selected => Style {
			day_background: palette.primary.strong.color.into(),
			text_color: palette.primary.strong.text,
			..base
		},
		Status::Hovered => Style {
			day_background: palette.primary.weak.color.into(),
			text_color: palette.primary.weak.text,
			..base
		},
		Status::Focused => Style {
			border_color: Color::from_rgb(0.5, 0.5, 0.5),
			..base
		},
		_ => base,
	}
}
