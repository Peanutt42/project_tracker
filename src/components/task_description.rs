use crate::{project_tracker::Message, styles::{markdown_background_container_style, markdown_style, PADDING_AMOUNT}, ProjectTrackerApp};
use iced::{widget::{container, markdown, text}, Element, Length::Fill};

pub fn generate_task_description_markdown(description: &str) -> Vec<markdown::Item> {
	markdown::parse(description).collect()
}

pub fn task_description<'a>(task_description_markdown_items: Option<&'a Vec<markdown::Item>>, app: &'a ProjectTrackerApp) -> Element<'a, Message> {
	container(
		match task_description_markdown_items {
			Some(task_description_markdown_items) if !task_description_markdown_items.is_empty() => {
				markdown(
					task_description_markdown_items,
					markdown::Settings::default(),
					markdown_style(app)
				)
				.map(|markdown_url| Message::OpenUrl(markdown_url.to_string()))
			},
			_ => text("No description")
				.width(Fill)
				.into(),
		}
	)
	.padding(PADDING_AMOUNT)
	.style(markdown_background_container_style)
	.into()
}