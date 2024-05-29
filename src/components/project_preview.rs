use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text}, Element, Length, Padding};
use iced_aw::ContextMenu;
use crate::{project::ProjectId, project_tracker::UiMessage, styles::SMALL_PADDING_AMOUNT};
use crate::components::{completion_bar, delete_project_button};
use crate::styles::{ContextMenuContainerStyle, ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, LIGHT_GREY, SMALL_HORIZONTAL_PADDING, PADDING_AMOUNT, SMALL_SPACING_AMOUNT};
use crate::project::Project;

pub fn project_preview(project: &Project, project_id: ProjectId, selected: bool) -> Element<UiMessage> {
	custom_project_preview(
		Some(project_id),
		project.get_completion_percentage(),
		project.get_tasks_done(),
		project.tasks.len(),
		text(&project.name)
			.size(LARGE_TEXT_SIZE)
			.into(),
		selected
	)
}

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
	.padding(SMALL_HORIZONTAL_PADDING);

	let underlay = 
		container(
				button(inner)
					.width(Length::Fill)
					.on_press_maybe(project_id.map(UiMessage::SelectProject))
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		)
		.padding(Padding{ right: PADDING_AMOUNT, ..Padding::ZERO });

	if let Some(project_id) = project_id {
		let context_overlay = move || {
			container(
				column![
					delete_project_button(project_id),
				]
			)
			.padding(Padding::new(SMALL_PADDING_AMOUNT))
			.style(theme::Container::Custom(Box::new(ContextMenuContainerStyle)))
			.into()
		};
	
		ContextMenu::new(underlay, context_overlay)
			.into()
	}
	else {
		underlay.into()
	}
}