use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text}, Color, Element, Length};
use crate::project_tracker::UiMessage;
use crate::components::completion_bar;
use crate::styles::ProjectPreviewButtonStyle;
use crate::project::Project;

pub fn project_preview(project: &Project, selected: bool) -> Element<UiMessage> {
	let inner = column![
		row![
			text(&project.name).size(20),
			container(
				text(format!("({}/{})", project.get_tasks_done(), project.tasks.len()))
					.style(theme::Text::Color(Color::from_rgb(0.75, 0.75, 0.75)))
					.size(13)
			)
			.width(Length::Fill)
			.align_x(Horizontal::Right),
		]
		.spacing(5),
		completion_bar(project.get_completion_percentage())
	];

	button(inner)
		.width(Length::Fill)
		.on_press(UiMessage::SelectProject(project.name.clone()))
		.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		.into()
}