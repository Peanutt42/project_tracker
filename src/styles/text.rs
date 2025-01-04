use crate::styles::GREY;
use iced::{
	font::{Family, Weight},
	widget::text::Style,
	Color, Font, Theme,
};

pub fn text_color(background: Color) -> Color {
	let brightness = 0.2126 * background.r + 0.7152 * background.g + 0.0722 * background.b;
	if brightness > 0.6 {
		Color::from_rgb(0.1, 0.1, 0.1)
	} else {
		Color::from_rgb(0.9, 0.9, 0.9)
	}
}

pub const FIRA_SANS_FONT: Font = Font::with_name("FiraSans");

pub const BOLD_FONT: Font = Font {
	weight: Weight::Bold,
	..Font::DEFAULT
};

pub const MONOSPACE_FONT: Font = Font {
	family: Family::Monospace,
	..Font::DEFAULT
};

pub fn default_text_style(_theme: &Theme) -> Style {
	Style { color: None }
}

pub fn danger_text_style(theme: &Theme) -> Style {
	Style {
		color: Some(theme.extended_palette().danger.base.color),
	}
}

pub fn grey_text_style(_theme: &Theme) -> Style {
	Style { color: Some(GREY) }
}
