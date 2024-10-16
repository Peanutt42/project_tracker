use iced::{widget::{container, markdown, text}, Element, Length::Fill};
use crate::{core::Task, project_tracker::Message, styles::{link_color, markdown_background_container_style, PADDING_AMOUNT}, ProjectTrackerApp};


pub fn task_description<'a>(task: &'a Task, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
	container(
		if task.description_markdown_items().is_empty() {
			text("No description")
				.width(Fill)
				.into()
		}
		else {
			markdown(
				task.description_markdown_items(),
				markdown::Settings::default(),
				markdown::Style {
					link_color: link_color(app.is_theme_dark()),
					..markdown::Style::from_palette(app.theme().palette())
				}
			)
			.map(|markdown_url| Message::OpenUrl(markdown_url.to_string()))
		}
	)
	.padding(PADDING_AMOUNT)
	.style(markdown_background_container_style)
	.into()
}