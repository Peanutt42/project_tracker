use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text}, Element, Length, Padding};
use iced_aw::ContextMenu;
use crate::{project_tracker::UiMessage, styles::SMALL_PADDING_AMOUNT};
use crate::components::{completion_bar, delete_project_button, move_project_up_button, move_project_down_button};
use crate::styles::{ContextMenuContainerStyle, ProjectPreviewButtonStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, LIGHT_GREY, SMALL_HORIZONTAL_PADDING, PADDING_AMOUNT, SMALL_SPACING_AMOUNT};
use crate::core::{Project, ProjectId};


pub fn project_preview(project: &Project, project_id: ProjectId, can_move_up: bool, can_move_down: bool, selected: bool) -> Element<UiMessage> {
	custom_project_preview(
		Some(project_id),
		can_move_up,
		can_move_down,
		project.get_completion_percentage(),
		project.get_tasks_done(),
		project.tasks.len(),
		text(&project.name)
			.size(LARGE_TEXT_SIZE)
			.into(),
		selected
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, can_move_up: bool, can_move_down: bool, project_completion_percentage: f32, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool) -> Element<UiMessage> {
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
			let mut context_buttons = column![
				delete_project_button(project_id),
			]
			.spacing(SMALL_SPACING_AMOUNT);
	
			if can_move_up {
				context_buttons = context_buttons.push(move_project_up_button(project_id));
			}
			if can_move_down {
				context_buttons = context_buttons.push(move_project_down_button(project_id));
			}

			container(context_buttons)
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