use iced::{alignment::Horizontal, theme, widget::{button, column, container, row, text, text_input, Row}, Alignment, Element, Length};
use once_cell::sync::Lazy;
use crate::{pages::SidebarPageMessage, project_tracker::UiMessage};
use crate::components::{completion_bar, edit_project_button, cancel_create_project_button, delete_project_button, move_project_up_button, move_project_down_button};
use crate::styles::{ProjectPreviewButtonStyle, TextInputStyle, SMALL_TEXT_SIZE, LARGE_TEXT_SIZE, LIGHT_GREY, SMALL_HORIZONTAL_PADDING, TINY_SPACING_AMOUNT, SMALL_SPACING_AMOUNT};
use crate::core::{Project, ProjectId};

pub static EDIT_PROJECT_NAME_TEXT_INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);

fn mouse_area<'a>(content: impl Into<Element<'a, UiMessage>>, project_id: Option<ProjectId>) -> Element<'a, UiMessage> {
	let mut mouse_area = iced::widget::mouse_area(content)
		.on_exit(SidebarPageMessage::MouseStoppedHoveringProject.into());

	if let Some(project_id) = project_id {
		mouse_area = mouse_area.on_move(move |_pos| SidebarPageMessage::MouseHoveredProject(project_id).into())
	}

	mouse_area.into()
}

pub fn project_preview(project: &Project, project_id: ProjectId, hovered: bool, editing: bool, can_move_up: bool, can_move_down: bool, selected: bool) -> Element<UiMessage> {
	let inner_text_element = if editing {
		text_input("project name", &project.name)
			.id(EDIT_PROJECT_NAME_TEXT_INPUT_ID.clone())
			.width(Length::Fill)
			.size(LARGE_TEXT_SIZE)
			.on_input(move |new_project_name| UiMessage::ChangeProjectName { project_id, new_project_name })
			.on_submit(SidebarPageMessage::StopEditingProject.into())
			.style(theme::TextInput::Custom(Box::new(TextInputStyle)))
			.into()
	}
	else {
		text(&project.name)
			.size(LARGE_TEXT_SIZE)
			.into()
	};

	custom_project_preview(
		Some(project_id),
		hovered,
		editing,
		can_move_up,
		can_move_down,
		project.get_completion_percentage(),
		project.get_tasks_done(),
		project.tasks.len(),
		inner_text_element,
		selected
	)
}

#[allow(clippy::too_many_arguments)]
pub fn custom_project_preview(project_id: Option<ProjectId>, hovered: bool, editing: bool, can_move_up: bool, can_move_down: bool, project_completion_percentage: f32, tasks_done: usize, task_len: usize, inner_text_element: Element<UiMessage>, selected: bool) -> Element<UiMessage> {
	let inner = column![
		row![
			inner_text_element,
			container(
				text(format!("({}/{})", tasks_done, task_len))
					.style(theme::Text::Color(LIGHT_GREY))
					.size(SMALL_TEXT_SIZE)
			)
			.width(if project_id.is_some() && !editing { Length::Fill } else { Length::Shrink })
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
					.on_press_maybe(project_id.map(UiMessage::SelectProject))
					.style(theme::Button::custom(ProjectPreviewButtonStyle{ selected }))
		)
		.width(Length::Fill);

	if let Some(project_id) = project_id {
		if editing {
			let move_project_element: Option<Element<UiMessage>> = {
				match (can_move_up, can_move_down) {
					(true, true) => Some(column![
						move_project_up_button(project_id),
						move_project_down_button(project_id),
					].into()),
					(true, false) => Some(move_project_up_button(project_id).into()),
					(false, true) => Some(move_project_down_button(project_id).into()),
					(false, false) => None,
				}
			};

			mouse_area(
				row![
					underlay,
					Row::new()
						.push_maybe(move_project_element)
						.push(delete_project_button(project_id))
						.spacing(SMALL_SPACING_AMOUNT)
						.align_items(Alignment::Center)
				]
				.align_items(Alignment::Center)
				.width(Length::Fill),

				Some(project_id),
			)
		}
		else {
			mouse_area(
				row![
					underlay,
					edit_project_button(project_id, hovered),
				]
				.align_items(Alignment::Center)
				.width(Length::Fill),

				Some(project_id),
			)
		}
	}
	else {
		mouse_area(
			row![
				underlay,
				cancel_create_project_button()
			]
			.align_items(Alignment::Center)
			.width(Length::Fill),

			None,
		)
	}
}
