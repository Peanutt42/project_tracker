use crate::{
	components::{
		days_left_widget, duration_widget, in_between_dropzone, open_in_code_editor_button,
		task_tag_button,
	},
	core::{View, TASK_TAG_QUAD_HEIGHT},
	icons::{icon_to_text, Bootstrap},
	integrations::CodeEditor,
	pages::sidebar_page,
	project_tracker::Message,
	styles::{
		checkbox_style, default_text_style, grey_text_style, rounded_container_style,
		task_background_container_style, task_button_style, PADDING_AMOUNT,
		SMALL_HORIZONTAL_PADDING, SMALL_PADDING_AMOUNT, SMALL_TEXT_SIZE, TINY_SPACING_AMOUNT,
	},
};
use iced::{
	alignment::Vertical,
	widget::{
		button, checkbox, column, container, container::Id, hover, row, text, Column, Row, Space,
	},
	Alignment, Element,
	Length::Fill,
	Padding, Size,
};
use iced_drop::droppable;
use project_tracker_core::{
	duration_str, round_duration_to_seconds, DatabaseMessage, Project, ProjectId, SortMode, Task,
	TaskId, TaskTagId, TaskType,
};
use std::{collections::BTreeSet, time::Duration};

#[allow(clippy::too_many_arguments)]
pub fn task_widget<'a>(
	task: &'a Task,
	task_id: TaskId,
	task_dropzone_id: Id,
	task_type: TaskType,
	project_id: ProjectId,
	project: &'a Project,
	code_editor: Option<&'a CodeEditor>,
	dragging: bool,
	just_minimal_dragging: bool,
	draggable: bool,
	highlight_dropzone: bool,
	show_due_date: bool,
	smaller_font: bool,
) -> Element<'a, Message> {
	let text_style = if matches!(task_type, TaskType::Done) {
		grey_text_style
	} else {
		default_text_style
	};

	let show_drag_grip = draggable && !dragging && matches!(project.sort_mode, SortMode::Manual);

	let on_hover_view: Element<'a, Message> = if show_drag_grip {
		let normal_grip_view = Element::new(icon_to_text(Bootstrap::GripVertical));

		container(match code_editor {
			Some(code_editor) if matches!(task_type, TaskType::SourceCodeTodo) => row![
				icon_to_text(Bootstrap::GripVertical),
				Space::new(Fill, 0.0),
				open_in_code_editor_button(task.description.clone(), code_editor),
			]
			.align_y(Vertical::Center)
			.padding(Padding::default().right(SMALL_PADDING_AMOUNT))
			.into(),
			_ => normal_grip_view,
		})
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
	} else {
		Space::new(0.0, 0.0).into()
	};

	let inner = |drag_overlay: bool| -> Element<'a, Message> {
		let inner_text = text(&task.name).width(Fill).style(text_style);
		let inner_text_element: Element<'a, Message> = if smaller_font {
			inner_text.size(15)
		} else {
			inner_text
		}
		.into();

		let tags_element = Row::with_children(
			project
				.task_tags
				.iter()
				.filter(|(tag_id, _tag)| task.tags.contains(tag_id))
				.map(|(_tag_id, tag)| tag.view()),
		)
		.spacing(TINY_SPACING_AMOUNT);

		let grip_icon_dummy: Element<Message> = if draggable {
			if dragging && matches!(project.sort_mode, SortMode::Manual) {
				container(icon_to_text(Bootstrap::GripVertical))
					.padding(Padding {
						top: if task.tags.is_empty() {
							0.0
						} else {
							TASK_TAG_QUAD_HEIGHT + TINY_SPACING_AMOUNT * 1.5
						},
						..Padding::ZERO
					})
					.into()
			} else {
				Space::new(PADDING_AMOUNT, 0.0).into()
			}
		} else {
			Space::new(0, 0).into()
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
						.push(inner_text_element) // text_editor leaves empty new line even if completly empty editor
						.push_maybe(if task.description.is_empty() || task.description == "\n" {
							None::<Element<'a, Message>>
						} else {
							Some(
								icon_to_text(Bootstrap::JustifyLeft)
									.size(SMALL_TEXT_SIZE)
									.into(),
							)
						})
						.spacing(TINY_SPACING_AMOUNT),
					Column::new().push_maybe(
						if task.needed_time_minutes.is_some()
							|| task.time_spend.is_some()
							|| task.due_date.is_some()
						{
							Some(
								Column::new()
									.push_maybe(
										match (task.needed_time_minutes, &task.time_spend) {
											(Some(needed_time_minutes), Some(time_spend)) => Some(
												container(text(format!(
													"{}/{}",
													duration_str(round_duration_to_seconds(
														time_spend.get_duration()
													)),
													duration_str(Duration::from_secs(
														needed_time_minutes as u64 * 60
													)),
												)))
												.padding(SMALL_HORIZONTAL_PADDING)
												.style(rounded_container_style)
												.into(),
											),
											(Some(needed_time_minutes), None) => {
												Some(duration_widget(Duration::from_secs(
													needed_time_minutes as u64 * 60,
												)))
											}
											(None, Some(time_spend)) => Some(
												container(text(format!(
													"{}/...",
													duration_str(round_duration_to_seconds(
														time_spend.get_duration()
													))
												)))
												.padding(SMALL_HORIZONTAL_PADDING)
												.style(rounded_container_style)
												.into(),
											),
											(None, None) => None,
										},
									)
									.push_maybe(if show_due_date {
										task.due_date.as_ref().map(|due_date| {
											days_left_widget(*due_date, task_type.is_done())
										})
									} else {
										None
									})
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
			.align_y(Vertical::Center),
		)
		.padding(Padding::new(SMALL_PADDING_AMOUNT))
		.style(move |t| {
			task_background_container_style(
				t,
				draggable && dragging && !just_minimal_dragging,
				drag_overlay,
			)
		})
		.into()
	};

	if draggable {
		column![
			in_between_dropzone(
				if matches!(task_type, TaskType::Todo) {
					task_dropzone_id.clone()
				} else {
					Id::unique()
				},
				highlight_dropzone
			),
			hover(
				droppable(inner(false))
					.on_drop(move |point, rect| sidebar_page::Message::DropTask {
						project_id,
						task_id,
						point,
						rect
					}
					.into())
					.on_click(Message::PressTask {
						project_id,
						task_id
					})
					.on_drag(move |point, rect| Message::DragTask {
						project_id,
						task_id,
						task_is_todo: matches!(task_type, TaskType::Todo),
						point,
						rect
					})
					.on_cancel(Message::CancelDragTask)
					.drag_overlay(!just_minimal_dragging)
					.override_overlay(
						container(inner(true)).center_y(Fill).into(),
						Some(Size::new(300.0, 80.0))
					)
					.drag_center(true)
					.drag_hide(false) //-  !just_minimal_dragging)
					.style(move |t, s| task_button_style(t, s, dragging && !just_minimal_dragging)),
				on_hover_view
			)
		]
		.into()
	} else {
		button(inner(false))
			.on_press(Message::OpenTaskModal {
				project_id,
				task_id,
			})
			.style(|t, s| task_button_style(t, s, false))
			.padding(Padding::ZERO)
			.into()
	}
}

/// Shows the enabled tags on the left first, then the disabled tags.
pub fn task_tag_list<'a>(
	project: &'a Project,
	enabled_tags: &'a BTreeSet<TaskTagId>,
	on_toggle: impl Fn(TaskTagId) -> Message,
) -> Vec<Element<'a, Message>> {
	let mut list = Vec::with_capacity(project.task_tags.len());
	for (tag_id, tag) in project.task_tags.iter() {
		if enabled_tags.contains(&tag_id) {
			list.push(
				task_tag_button(tag, enabled_tags.contains(&tag_id))
					.on_press(on_toggle(tag_id))
					.into(),
			);
		}
	}
	for (tag_id, tag) in project.task_tags.iter() {
		if !enabled_tags.contains(&tag_id) {
			list.push(
				task_tag_button(tag, enabled_tags.contains(&tag_id))
					.on_press(on_toggle(tag_id))
					.into(),
			);
		}
	}
	list
}
