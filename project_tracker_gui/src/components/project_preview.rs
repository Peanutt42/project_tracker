use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text, text_input}, Alignment, Element, Length};
use once_cell::sync::Lazy;
use crate::project_tracker::UiMessage;
use crate::components::{completion_bar, cancel_create_project_button};
use crate::styles::{ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, LIGHT_GREY, SMALL_HORIZONTAL_PADDING, TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT};
use crate::core::{Project, ProjectId};

pub static EDIT_PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

pub fn project_preview(project: &Project, project_id: ProjectId, selected: bool) -> Element<UiMessage> {
	let inner_text_element = text(&project.name).size(LARGE_TEXT_SIZE).into();

	custom_project_preview(
		Some(project_id),
		project.get_completion_percentage(),
		project.get_tasks_done(),
		project.tasks.len(),
		inner_text_element,
		selected
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, project_completion_percentage: f32, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool) -> Element<UiMessage> {
	let inner = column![
		row![
			inner_text_element,
			container(
				text(format!("({}/{})", tasks_done, task_len))
					.style(theme::Text::Color(LIGHT_GREY))
					.size(SMALL_TEXT_SIZE)
			)
			.width(if project_id.is_some() { Length::Fill } else { Length::Shrink })
			.align_x(Horizontal::Right),
		]
		.width(Length::Fill)
		.spacing(SMALL_SPACING_AMOUNT),

		completion_bar(project_completion_percentage)
	]
	.padding(SMALL_HORIZONTAL_PADDING)
	.spacing(TINY_SPACING_AMOUNT);

	let underlay =
		container(
				button(inner)
					.width(Length::Fill)
					.on_press(UiMessage::SelectProject(project_id))
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		)
		.width(Length::Fill);

	if project_id.is_some() {
		underlay.into()
	}
	else {
		row![
			underlay,
			cancel_create_project_button()
		]
		.align_items(Alignment::Center)
		.spacing(SMALL_SPACING_AMOUNT)
		.width(Length::Fill)
   		.into()
	}
}
