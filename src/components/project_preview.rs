use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text}, Color, Element, Length};
use crate::{project_tracker::UiMessage, styles::SMALL_HORIZONTAL_PADDING};
use crate::components::completion_bar;
use crate::styles::{ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, SMALL_SPACING_AMOUNT};
use crate::project::Project;

pub fn project_preview(project: &Project, selected: bool) -> Element<UiMessage> {
	let inner = column![
		row![
			text(&project.name).size(LARGE_TEXT_SIZE),
			container(
				text(format!("({}/{})", project.get_tasks_done(), project.tasks.len()))
					.style(theme::Text::Color(Color::from_rgb(0.75, 0.75, 0.75)))
					.size(SMALL_TEXT_SIZE)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Right),
		]
		.spacing(SMALL_SPACING_AMOUNT),
		completion_bar(project.get_completion_percentage())
	]
	.padding(SMALL_HORIZONTAL_PADDING);

	button(inner)
		.width(Length::Fill)
		.on_press(UiMessage::SelectProject(project.name.clone()))
		.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		.into()
}