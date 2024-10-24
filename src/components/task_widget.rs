use crate::{
	components::{days_left_widget, duration_widget, in_between_dropzone
	}, core::{
		DatabaseMessage, Project, ProjectId, SortMode, Task, TaskId, TaskType, TASK_TAG_QUAD_HEIGHT
	}, icons::{icon_to_text, Bootstrap}, pages::{ProjectPageMessage, SidebarPageMessage}, project_tracker::Message, styles::{
		checkbox_style, secondary_button_style_default, task_background_container_style, task_button_style, GREY, PADDING_AMOUNT, SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_TEXT_SIZE, TINY_SPACING_AMOUNT
	}
};
use iced::widget::{hover, Space};
use iced::{
	alignment::Vertical,
	widget::{
		button, checkbox, column, container, container::Id, row, text,
		Column, Row,
	},
	Alignment, Element,
	Length::Fill,
	Padding, Theme,
};
use iced_drop::droppable;
use std::{borrow::Cow, time::Duration};

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(
	task: &'a Task,
	task_id: TaskId,
	task_type: TaskType,
	project_id: ProjectId,
	project: &'a Project,
	dragging: bool,
	just_minimal_dragging: bool,
	highlight: bool,
	stopwatch_label: Option<&'a String>
) -> Element<'a, Message> {
	let tags_element = Row::with_children(
		project.task_tags
			.iter()
			.filter(|(tag_id, _tag)| task.tags.contains(tag_id))
			.map(|(_tag_id, tag)| tag.view()),
	)
	.spacing(TINY_SPACING_AMOUNT);

	let text_style = if matches!(task_type, TaskType::Done) {
		text::Style { color: Some(GREY) }
	}
	else {
		text::Style::default()
	};

	// TODO: how should markdown look on done tasks?
	let inner_text_element: Element<'a, Message> =
		text(task.name())
			.width(Fill)
			.style(move |_| text_style)
			.into();

	let show_drag_grip = !dragging && matches!(project.sort_mode, SortMode::Manual);

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

	let grip_icon_dummy: Element<Message> = if dragging && matches!(project.sort_mode, SortMode::Manual) {
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

	let inner: Element<'a, Message> = container(
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
					.push_maybe(if task.description_markdown_items().is_empty() {
						None::<Element<'a, Message>>
					}
					else {
						Some(
							icon_to_text(Bootstrap::JustifyLeft)
								.size(SMALL_TEXT_SIZE)
								.into()
						)
					})
					.spacing(TINY_SPACING_AMOUNT),

				Column::new().push_maybe(
					if task.needed_time_minutes.is_some() || task.due_date.is_some() {
						Some(
							Column::new()
								.push_maybe(task.needed_time_minutes.map(|duration_minutes| {
									duration_widget(Cow::Owned(Duration::from_secs(
										duration_minutes as u64 * 60,
									)))
								}))
								.push_maybe(
									task.due_date
										.as_ref()
										.map(|due_date| days_left_widget(*due_date)),
								)
								.push_maybe(stopwatch_label.map(|label| -> Element<Message> {
									button(
										row![
											icon_to_text(Bootstrap::Stopwatch).size(SMALL_TEXT_SIZE),
											text(label).style(|theme: &Theme| text::Style {
												color: Some(theme.extended_palette().danger.base.color)
											})
										]
										.align_y(Vertical::Center)
										.spacing(TINY_SPACING_AMOUNT),
									)
									.padding(SMALL_HORIZONTAL_PADDING)
									.style(secondary_button_style_default)
									.on_press(Message::OpenStopwatch)
									.into()
								}))
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
		dragging && !just_minimal_dragging
	))
	.into();

	column![
		in_between_dropzone(
			if matches!(task_type, TaskType::Todo) {
				task.dropzone_id.clone()
			} else {
				Id::unique()
			},
			highlight
		),
		match &project.sort_mode {
			SortMode::Manual => hover(
				droppable(
					inner
				)
				.on_drop(move |point, rect| SidebarPageMessage::DropTask {
					project_id,
					task_id,
					point,
					rect
				}
				.into())
				.on_click(ProjectPageMessage::PressTask(task_id).into())
				.on_drag(move |point, rect| Message::DragTask {
					project_id,
					task_id,
					task_is_todo: matches!(task_type, TaskType::Todo),
					point,
					rect
				})
				.on_cancel(Message::CancelDragTask)
				.drag_overlay(!just_minimal_dragging)
				.drag_hide(!just_minimal_dragging)
				.style(task_button_style),

				on_hover_view
			),
			_ => button(
				inner
			)
			.on_press(ProjectPageMessage::OpenTask(task_id).into())
			.style(task_button_style)
			.padding(Padding::ZERO)
			.into(),
		}
	]
	.into()
}