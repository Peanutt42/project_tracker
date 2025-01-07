use crate::styles::{link_color, BORDER_RADIUS};
use crate::ProjectTrackerApp;
use iced::{
	border::rounded,
	widget::markdown::{Highlight, Style},
	Color,
};

pub fn markdown_style(app: &ProjectTrackerApp) -> Style {
	Style {
		link_color: link_color(app.is_theme_dark()),
		inline_code_highlight: Highlight {
			background: Color::from_rgb8(17, 17, 17).into(),
			border: rounded(BORDER_RADIUS),
		},
		..Style::from_palette(app.theme().palette())
	}
}
