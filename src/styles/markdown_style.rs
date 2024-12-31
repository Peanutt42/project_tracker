use crate::styles::link_color;
use crate::ProjectTrackerApp;
use iced::widget::markdown::Style;

pub fn markdown_style(app: &ProjectTrackerApp) -> Style {
	Style {
		link_color: link_color(app.is_theme_dark()),
		..Style::from_palette(app.theme().palette())
	}
}
