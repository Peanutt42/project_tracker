use iced::{border::rounded, widget::progress_bar::Style, Theme};
use crate::styles::color_average;

pub fn completion_bar_style(theme: &Theme) -> Style {
	Style {
		background: color_average(
			theme.extended_palette().background.weak.color,
			theme.extended_palette().background.base.color
		)
		.into(),
		bar: theme.extended_palette().primary.base.color.into(),
		border: rounded(2.5)
	}
}