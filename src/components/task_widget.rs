use crate::{
	components::{days_left_widget, duration_str, duration_widget, in_between_dropzone, round_duration_to_seconds}, core::{View, TASK_TAG_QUAD_HEIGHT}, icons::{icon_to_text, Bootstrap}, modals::TaskModalMessage, pages::SidebarPageMessage, project_tracker::Message, styles::{checkbox_style, default_text_style, grey_text_style, rounded_container_style, task_background_container_style, task_button_style, PADDING_AMOUNT, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_TEXT_SIZE, TINY_SPACING_AMOUNT}
};
use project_tracker_core::{DatabaseMessage, Project, ProjectId, SortMode, Task, TaskId, TaskType};
use iced::widget::{hover, markdown, Space};
use iced::{
	alignment::Vertical,
	widget::{
		button, checkbox, column, container, container::Id, row, text,
		Column, Row,
	},
	Alignment, Element,
	Length::Fill,
	Padding,
};
use iced_drop::droppable;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(
	task: &'a Task,
	task_id: TaskId,
	task_dropzone_id: Id,
	task_type: TaskType,
	task_description_markdown_items: Option<&'a Vec<markdown::Item>>,
	project_id: ProjectId,
	project: &'a Project,
	dragging: bool,
	just_minimal_dragging: bool,
	draggable: bool,
	highlight_dropzone: bool,
	show_due_date: bool
) -> Element<'a, Message> {
	let text_style = if matches!(task_type, TaskType::Done) {
		grey_text_style
	}
	else {
		default_text_style
	};

	let show_drag_grip = draggable && !dragging && matches!(project.sort_mode, SortMode::Manual);

	let on_hover_view: Element<'a, Message> = if show_drag_grip {
		container(icon_to_text(Bootstrap::GripVertical))
			.padding(Padding {
				top: if task.tags.is_empty() {
					0.0
				} else {
					TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT * 1.5
				},
				..Padding::ZERO
			})
			.center_y(Fill)
			.into()
	}
	else {
		Space::new(0.0, 0.0).into()
	};

	let inner = |drag_overlay: bool| -> Element<'a, Message> {
		let inner_text_element: Element<'a, Message> =
			text(task.name())
				.width(Fill)
				.style(text_style)
				.into();

		let tags_element = Row::with_children(
			project.task_tags
				.iter()
				.filter(|(tag_id, _tag)| task.tags.contains(tag_id))
				.map(|(_tag_id, tag)| tag.view()),
		)
		.spacing(TINY_SPACING_AMOUNT);

		let grip_icon_dummy: Element<Message> = if draggable && dragging && matches!(project.sort_mode, SortMode::Manual) {
			container(
				icon_to_text(Bootstrap::GripVertical)
			)
			.padding(Padding {
				top: if task.tags.is_empty() {
					0.0
				} else {
					TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT * 1.5
				},
				..Padding::ZERO
			})
			.into()
		}
		else {
			Space::new(PADDING_AMOUNT, 0.0).into()
		};

		container(
			row![
				grip_icon_dummy,
				row![
					container(
						checkbox("", matches!(task_type, TaskType::Done))
							.on_toggle(move |checked| {
								if checked {
									DatabaseMessage::SetTaskDone {
										project_id,
										task_id,
									}
									.into()
								} else {
									DatabaseMessage::SetTaskTodo {
										project_id,
										task_id,
									}
									.into()
								}
							})
							.style(checkbox_style)
					)
					.padding(Padding {
						top: if task.tags.is_empty() {
							0.0
						} else {
							TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT
						},
						..Padding::ZERO
					}),

					Column::new()
						.push_maybe(if task.tags.is_empty() {
							None
						} else {
							Some(tags_element)
						})
						.push(inner_text_element)
						.push_maybe(match task_description_markdown_items {
							Some(task_description_markdown_items) if !task_description_markdown_items.is_empty() => {
								Some(
									icon_to_text(Bootstrap::JustifyLeft)
										.size(SMALL_TEXT_SIZE)
										.into()
								)
							},
							_ => None::<Element<'a, Message>>
						})
						.spacing(TINY_SPACING_AMOUNT),

					Column::new().push_maybe(
						if task.needed_time_minutes.is_some() || task.time_spend.is_some() || task.due_date.is_some() {
							Some(
								Column::new()
									.push_maybe(
										match (task.needed_time_minutes, &task.time_spend) {
											(Some(needed_time_minutes), Some(time_spend)) => Some(
												container(
													text(
														format!(
															"{}/{}",
															duration_str(round_duration_to_seconds(time_spend.get_duration())),
															duration_str(Duration::from_secs(needed_time_minutes as u64 * 60)),
														)
													)
												)
												.padding(SMALL_HORIZONTAL_PADDING)
												.style(rounded_container_style)
												.into()
											),
											(Some(needed_time_minutes), None) => Some(
												duration_widget(Duration::from_secs(needed_time_minutes as u64 * 60))
											),
											(None, Some(time_spend)) => Some(
												container(
													text(format!(
														"{}/...",
														duration_str(round_duration_to_seconds(time_spend.get_duration()))
													))
												)
												.padding(SMALL_HORIZONTAL_PADDING)
												.style(rounded_container_style)
												.into()
											),
											(None, None) => None,
										}
									)
									.push_maybe(
										if show_due_date {
											task.due_date
												.as_ref()
												.map(|due_date| days_left_widget(*due_date, task_type.is_done()))
										} else {
											None
										}
									)
									.spacing(TINY_SPACING_AMOUNT)
									.align_x(Alignment::End)
									.into(),
							) as Option<Element<Message>>
						} else {
							None
						}
					),
				]
				.width(Fill)
				.align_y(Alignment::Start)
			]
			.align_y(Vertical::Center)
		)
		.padding(Padding::new(SMALL_PADDING_AMOUNT))
		.style(move |t| task_background_container_style(
			t,
			draggable && dragging && !just_minimal_dragging,
			drag_overlay
		))
		.into()
	};

	column![
		in_between_dropzone(
			if matches!(task_type, TaskType::Todo) {
				task_dropzone_id.clone()
			} else {
				Id::unique()
			},
			highlight_dropzone
		),
		if draggable {
			hover(
				droppable(
					inner(false)
				)
				.on_drop(move |point, rect| SidebarPageMessage::DropTask {
					project_id,
					task_id,
					point,
					rect
				}
				.into())
				.on_click(Message::PressTask{ project_id, task_id })
				.on_drag(move |point, rect| Message::DragTask {
					project_id,
					task_id,
					task_is_todo: matches!(task_type, TaskType::Todo),
					point,
					rect
				})
				.on_cancel(Message::CancelDragTask)
				.drag_overlay(!just_minimal_dragging, Some(inner(true)))
				.drag_hide(false)//-  !just_minimal_dragging)
				.style(move |t, s| task_button_style(t, s, dragging && !just_minimal_dragging)),

				on_hover_view
			)
		} else {
			button(
				inner(false)
			)
			.on_press(TaskModalMessage::Open{ project_id, task_id }.into())
			.style(|t, s| task_button_style(t, s, false))
			.padding(Padding::ZERO)
			.into()
		}
	]
	.into()
}